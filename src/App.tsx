import { createMemo, createResource, createEffect, Show, For } from "solid-js";
import "@total-typescript/ts-reset";

import type { Component } from "solid-js";

import config from "./config.js";
import * as st from "spacetraders-sdk";
import * as hexastore from "./hexastore.js";

import type { Ship } from "spacetraders-sdk";

const percent = (numerator: number, denominator: number) =>
  Math.floor((numerator / denominator) * 100) + "%";

const [ships, { refetch: refreshShips }] = createResource(async () => {
  const fleet = new st.FleetApi(config);
  const { data: ships } = await fleet.getMyShips();
  console.log(ships);
  return ships;
});

setInterval(() => refreshShips(), 2_000);

const [contracts, { refetch: refreshContracts }] = createResource(async () => {
  const api = new st.ContractsApi(config);
  const { data: contracts } = await api.getContracts();
  return contracts;
});

const cargoContractOverlap = (ship: Ship) => {
  if (!contracts()) return [];
  const cargoItems = ship.cargo.inventory.map((item) => item.symbol);
  const contractedItems = contracts()!
    .flatMap((contract) =>
      contract.terms.deliver
        ?.filter((item) => item.unitsFulfilled < item.unitsRequired)
        .map((item) => ({
          symbol: item.tradeSymbol,
          destination: item.destinationSymbol,
          contract: contract.id,
        }))
    )
    .filter(Boolean);
  return contractedItems.filter(
    (item) => cargoItems.indexOf(item.symbol) !== -1
  );
};

const sellCargo = async (ship: Ship) => {
  const fleet = new st.FleetApi(config);
  const contractedCargo = cargoContractOverlap(ship);
  for (let contractedItem of contractedCargo) {
    if (ship.nav.waypointSymbol !== contractedItem.destination) {
      const { data: navigation } = await fleet.navigateShip({
        shipSymbol: ship.symbol,
        navigateShipRequest: { waypointSymbol: contractedItem.destination },
      });
      const scheduledArrival = navigation.nav.route.arrival;
      await new Promise((resolve) =>
        setTimeout(resolve, scheduledArrival.getTime() - new Date().getTime())
      );
    }
    await fleet.dockShip({ shipSymbol: ship.symbol });

    await fleet.sellCargo({
      shipSymbol: ship.symbol,
      sellCargoRequest: {
        symbol: contractedItem.symbol,
        units: ship.cargo.inventory.find(
          (item) => item.symbol === contractedItem.symbol
        )!.units,
      },
    });
  }
  const leftoverCargo = ship.cargo.inventory.filter(
    (item) =>
      contractedCargo.find((i) => i.symbol === item.symbol) === undefined
  );
  for (let item of leftoverCargo) {
    await fleet.sellCargo({
      shipSymbol: ship.symbol,
      sellCargoRequest: { symbol: item.symbol, units: item.units },
    });
  }
  console.log("Sold!");
};

const Transit: Component<{ ship: Ship }> = (props) => {
  const nav = createMemo(() => props.ship.nav);
  return (
    <Show when={nav().status === "IN_TRANSIT"}>
      Arriving in{" "}
      {Math.round(
        (nav().route.arrival.getTime() - new Date().getTime()) / 1_000
      )}{" "}
      seconds
    </Show>
  );
};

const NavigateShip: Component<{ ship: Ship }> = (props) => {
  const fleet = new st.FleetApi(config);

  const [waypoints] = createResource(
    async (): Promise<st.ScannedWaypoint[]> => {
      const system = props.ship.nav.systemSymbol;
      const subject = `system:${system}`;
      const predicate = "canReach";
      const cached = await hexastore.db
        .values<string, st.ScannedWaypoint>(
          hexastore.sp({ subject, predicate })
        )
        .all();
      if (cached.length) return cached;

      const { data: availableWaypoints } = await fleet.createShipWaypointScan({
        shipSymbol: props.ship.symbol,
      });
      await hexastore.db.batch(
        availableWaypoints.waypoints.flatMap((waypoint) =>
          hexastore.batch(
            "put",
            { subject, predicate, object: waypoint.symbol },
            waypoint
          )
        )
      );
      return availableWaypoints.waypoints;
    }
  );

  const go = async (waypoint: st.ScannedWaypoint) => {
    const fleet = new st.FleetApi(config);
    await fleet.navigateShip({
      shipSymbol: props.ship.symbol,
      navigateShipRequest: { waypointSymbol: waypoint.symbol },
    });
    console.log("Navigating!");
    await refreshShips();
  };

  return (
    <>
      <header>Available waypoints:</header>
      <ul>
        <For each={waypoints() ?? []} fallback={<p>No waypoints available.</p>}>
          {(waypoint) => (
            <li>
              {waypoint.symbol} ({waypoint.type}){" "}
              <button type="button" onclick={() => go(waypoint)}>
                ➡️
              </button>
              <ul class="flex flex-wrap gap-2 list-disc list-inside">
                {waypoint.traits.map((trait) => (
                  <li class="text-xs italic">{trait.symbol}</li>
                ))}
              </ul>
            </li>
          )}
        </For>
      </ul>
    </>
  );
};

const OrbitOrDock: Component<{ ship: Ship }> = (props) => {
  const orbitOrDock = async () => {
    const fleet = new st.FleetApi(config);
    if (props.ship.nav.status === "IN_TRANSIT") return;
    const fn =
      props.ship.nav.status === "DOCKED"
        ? fleet.orbitShip.bind(fleet)
        : fleet.dockShip.bind(fleet);
    await fn({ shipSymbol: props.ship.symbol });
    await refreshShips();
  };

  return (
    <Show when={props.ship.nav.status !== "IN_TRANSIT"}>
      {props.ship.nav.status} =&gt;{" "}
      <button
        class="px-2 py-1 rounded-md bg-white border-1"
        type="button"
        onclick={() => orbitOrDock()}
      >
        {props.ship.nav.status === "DOCKED" ? "ORBIT" : "DOCK"}
      </button>
    </Show>
  );
};

const ShipList: Component = () => {
  return (
    <Show when={ships()} fallback={<p>loading</p>}>
      {(ships) => (
        <ul>
          <For each={ships()}>
            {(ship) => (
              <>
                <li class="mb-4 last:mb-0">
                  <header>
                    {ship.symbol}{" "}
                    <span class="text-sm italic">
                      ({ship.registration.role.toLowerCase()})
                    </span>
                  </header>

                  <p>
                    <span>
                      {ship.nav.route.destination.type}{" "}
                      {ship.nav.waypointSymbol}
                    </span>
                    &nbsp;
                    <OrbitOrDock ship={ship} />
                    <span class="text-sm italic">({ship.nav.flightMode})</span>
                    <Transit ship={ship} />
                  </p>

                  <dl>
                    <For
                      each={[
                        [
                          "Cargo",
                          percent(ship.cargo.units, ship.cargo.capacity),
                        ],
                        [
                          "Fuel",
                          percent(ship.fuel.current, ship.fuel.capacity),
                        ],
                      ]}
                    >
                      {([k, v]) => (
                        <div>
                          <dt class="inline mr-2">{k}</dt>
                          <dd class="inline italic">{v}</dd>
                          <hr />
                        </div>
                      )}
                    </For>
                  </dl>
                  <button
                    type="button"
                    class="bg-white"
                    onclick={() => sellCargo(ship)}
                  >
                    Sell cargo
                  </button>
                </li>

                <ul>
                  <For each={ship.cargo.inventory}>
                    {(item) => (
                      <li>
                        {item.symbol} ({item.units})
                      </li>
                    )}
                  </For>
                </ul>
                <NavigateShip ship={ship} />
              </>
            )}
          </For>
        </ul>
      )}
    </Show>
  );
};

const Navigate: Component = () => {
  let shipEl: HTMLSelectElement | undefined;
  let destinationEl: HTMLInputElement | undefined;

  const onsubmit = async (event: Event) => {
    event.preventDefault();
    const fleet = new st.FleetApi(config);
    await fleet.navigateShip({
      shipSymbol: shipEl!.value,
      navigateShipRequest: { waypointSymbol: destinationEl!.value },
    });
    console.log("Navigating!");
    await refreshShips();
  };

  return (
    <form onsubmit={onsubmit}>
      <label>
        Ship
        <select ref={shipEl}>
          <For each={ships() ?? []}>
            {(ship) => <option value={ship.symbol}>{ship.symbol}</option>}
          </For>
        </select>
      </label>

      <label>
        Destination <input type="text" ref={destinationEl} required />
      </label>
      <button type="submit" onsubmit={onsubmit}>
        Navigate
      </button>
    </form>
  );
};

const App: Component = () => {
  return (
    <div class="px-8 py-4 min-h-screen bg-blue-300">
      <div class="bg-blue-50 rounded-xl p-4">
        <ShipList />
        <hr />
        <Navigate />
      </div>
    </div>
  );
};

export default App;
