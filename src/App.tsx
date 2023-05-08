import { createResource, Show, For } from "solid-js";
import type { Component } from "solid-js";

import config from "./config.js";
import * as st from "spacetraders-sdk";

const percent = (numerator: number, denominator: number) =>
  Math.floor((numerator / denominator) * 100) + "%";

const ShipList: Component = () => {
  const [ships, { mutate, refetch }] = createResource(async () => {
    const fleet = new st.FleetApi(config);
    const { data: ships } = await fleet.getMyShips();
    console.log(ships);
    return ships;
  });

  return (
    <Show when={ships()} fallback={<p>loading</p>}>
      {(ships) => (
        <ul>
          <For each={ships()}>
            {(ship) => (
              <li class="mb-4 last:mb-0">
                <header>
                  {ship.symbol}{" "}
                  <span class="text-sm italic">
                    ({ship.registration.role.toLowerCase()})
                  </span>
                </header>

                <p>
                  {ship.nav.status} at {ship.nav.waypointSymbol}{" "}
                  <span class="text-sm italic">({ship.nav.flightMode})</span>
                </p>

                <dl>
                  <For
                    each={[
                      ["Cargo", percent(ship.cargo.units, ship.cargo.capacity)],
                      ["Fuel", percent(ship.fuel.current, ship.fuel.capacity)],
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
              </li>
            )}
          </For>
        </ul>
      )}
    </Show>
  );
};

const App: Component = () => {
  return (
    <div class="px-8 py-4 min-h-screen bg-blue-300">
      <div class="bg-blue-50 rounded-xl p-4">
        <ShipList />
      </div>
    </div>
  );
};

export default App;
