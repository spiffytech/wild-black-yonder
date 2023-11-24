/*
 * SpaceTraders API
 *
 * SpaceTraders is an open-universe game and learning platform that offers a set of HTTP endpoints to control a fleet of ships and explore a multiplayer universe.  The API is documented using [OpenAPI](https://github.com/SpaceTradersAPI/api-docs). You can send your first request right here in your browser to check the status of the game server.  ```json http {   \"method\": \"GET\",   \"url\": \"https://api.spacetraders.io/v2\", } ```  Unlike a traditional game, SpaceTraders does not have a first-party client or app to play the game. Instead, you can use the API to build your own client, write a script to automate your ships, or try an app built by the community.  We have a [Discord channel](https://discord.com/invite/jh6zurdWk5) where you can share your projects, ask questions, and get help from other players.   
 *
 * The version of the OpenAPI document: 2.0.0
 * Contact: joel@spacetraders.io
 * Generated by: https://openapi-generator.tech
 */

/// ShipModule : A module can be installed in a ship and provides a set of capabilities such as storage space or quarters for crew. Module installations are permanent.



#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ShipModule {
    /// The symbol of the module.
    #[serde(rename = "symbol")]
    pub symbol: Symbol,
    /// Modules that provide capacity, such as cargo hold or crew quarters will show this value to denote how much of a bonus the module grants.
    #[serde(rename = "capacity", skip_serializing_if = "Option::is_none")]
    pub capacity: Option<i32>,
    /// Modules that have a range will such as a sensor array show this value to denote how far can the module reach with its capabilities.
    #[serde(rename = "range", skip_serializing_if = "Option::is_none")]
    pub range: Option<i32>,
    /// Name of this module.
    #[serde(rename = "name")]
    pub name: String,
    /// Description of this module.
    #[serde(rename = "description")]
    pub description: String,
    #[serde(rename = "requirements")]
    pub requirements: Box<crate::models::ShipRequirements>,
}

impl ShipModule {
    /// A module can be installed in a ship and provides a set of capabilities such as storage space or quarters for crew. Module installations are permanent.
    pub fn new(symbol: Symbol, name: String, description: String, requirements: crate::models::ShipRequirements) -> ShipModule {
        ShipModule {
            symbol,
            capacity: None,
            range: None,
            name,
            description,
            requirements: Box::new(requirements),
        }
    }
}

/// The symbol of the module.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
pub enum Symbol {
    #[serde(rename = "MODULE_MINERAL_PROCESSOR_I")]
    MineralProcessorI,
    #[serde(rename = "MODULE_GAS_PROCESSOR_I")]
    GasProcessorI,
    #[serde(rename = "MODULE_CARGO_HOLD_I")]
    CargoHoldI,
    #[serde(rename = "MODULE_CARGO_HOLD_II")]
    CargoHoldIi,
    #[serde(rename = "MODULE_CARGO_HOLD_III")]
    CargoHoldIii,
    #[serde(rename = "MODULE_CREW_QUARTERS_I")]
    CrewQuartersI,
    #[serde(rename = "MODULE_ENVOY_QUARTERS_I")]
    EnvoyQuartersI,
    #[serde(rename = "MODULE_PASSENGER_CABIN_I")]
    PassengerCabinI,
    #[serde(rename = "MODULE_MICRO_REFINERY_I")]
    MicroRefineryI,
    #[serde(rename = "MODULE_ORE_REFINERY_I")]
    OreRefineryI,
    #[serde(rename = "MODULE_FUEL_REFINERY_I")]
    FuelRefineryI,
    #[serde(rename = "MODULE_SCIENCE_LAB_I")]
    ScienceLabI,
    #[serde(rename = "MODULE_JUMP_DRIVE_I")]
    JumpDriveI,
    #[serde(rename = "MODULE_JUMP_DRIVE_II")]
    JumpDriveIi,
    #[serde(rename = "MODULE_JUMP_DRIVE_III")]
    JumpDriveIii,
    #[serde(rename = "MODULE_WARP_DRIVE_I")]
    WarpDriveI,
    #[serde(rename = "MODULE_WARP_DRIVE_II")]
    WarpDriveIi,
    #[serde(rename = "MODULE_WARP_DRIVE_III")]
    WarpDriveIii,
    #[serde(rename = "MODULE_SHIELD_GENERATOR_I")]
    ShieldGeneratorI,
    #[serde(rename = "MODULE_SHIELD_GENERATOR_II")]
    ShieldGeneratorIi,
}

impl Default for Symbol {
    fn default() -> Symbol {
        Self::MineralProcessorI
    }
}

