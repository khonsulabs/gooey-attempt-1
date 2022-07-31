use bonsaidb::core::{
    api::{Api, Infallible},
    schema::{ApiName, Qualified},
};
use serde::{Deserialize, Serialize};

/// The name of the database that the counter will use.
pub const DATABASE_NAME: &str = "counter";

/// Increments the counter.
#[derive(Serialize, Deserialize, Debug)]
pub struct IncrementCounter;

/// The current value of the counter. Sent whenever requested or when the counter is updated.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, Default)]
pub struct CounterValue(pub u64);

impl Api for IncrementCounter {
    type Error = Infallible;
    type Response = CounterValue;

    fn name() -> ApiName {
        ApiName::private("increment-counter")
    }
}

impl Api for CounterValue {
    type Error = Infallible;
    type Response = CounterValue;

    fn name() -> ApiName {
        ApiName::private("counter-value")
    }
}
