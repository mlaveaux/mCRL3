//! Parses and checks every `.stark` file under `examples/stark/`.
//!
//! Each file is exercised end-to-end: parse into an [UntypedStarkSpecification],
//! then [UntypedStarkSpecification::check] (name resolution + type checking).

use merc_stark::UntypedStarkSpecification;
use test_case::test_case;

#[test_case(include_str!("../../../examples/stark/engine.stark") ; "engine.stark")]
#[test_case(include_str!("../../../examples/stark/random_walk.stark") ; "random_walk.stark")]
#[test_case(include_str!("../../../examples/stark/single_vehicle.stark") ; "single_vehicle.stark")]
#[test_case(include_str!("../../../examples/stark/toll.stark") ; "toll.stark")]
#[test_case(include_str!("../../../examples/stark/two_vehicles.stark") ; "two_vehicles.stark")]
#[test_case(include_str!("../../../examples/stark/monitoring.stark") ; "monitoring.stark")]
#[test_case(include_str!("../../../examples/stark/agriculturalDT.stark") ; "agriculturalDT.stark")]
#[test_case(include_str!("../../../examples/stark/tollbooth.stark") ; "tollbooth.stark")]
#[test_case(include_str!("../../../examples/stark/engine_full.stark") ; "engine_full.stark")]
#[test_case(include_str!("../../../examples/stark/isocitrate.stark") ; "isocitrate.stark")]
#[test_case(include_str!("../../../examples/stark/envzompr.stark") ; "envzompr.stark")]
#[test_case(include_str!("../../../examples/stark/vehicle_full.stark") ; "vehicle_full.stark")]
#[test_case(include_str!("../../../examples/stark/multiscler.stark") ; "multiscler.stark")]
#[test_case(include_str!("../../../examples/stark/lotka.stark") ; "lotka.stark")]
#[test_case(include_str!("../../../examples/stark/polistil.stark") ; "polistil.stark")]
#[test_case(include_str!("../../../examples/stark/turtle.stark") ; "turtle.stark")]
#[test_case(include_str!("../../../examples/stark/turtle_hospital.stark") ; "turtle_hospital.stark")]
#[test_case(include_str!("../../../examples/stark/repressilator.stark") ; "repressilator.stark")]
#[test_case(include_str!("../../../examples/stark/reactionsystems_running.stark") ; "reactionsystems_running.stark")]
#[test_case(include_str!("../../../examples/stark/reactionsystems_lacoperon.stark") ; "reactionsystems_lacoperon.stark")]
#[test_case(include_str!("../../../examples/stark/reactionsystems_synapse.stark") ; "reactionsystems_synapse.stark")]
#[test_case(include_str!("../../../examples/stark/reactionsystems_synapse_3neuron.stark") ; "reactionsystems_synapse_3neuron.stark")]
#[test_case(include_str!("../../../examples/stark/abz2025_single_lane_two_cars.stark") ; "abz2025_single_lane_two_cars.stark")]
#[test_case(include_str!("../../../examples/stark/abz2025_one_lane_three_cars.stark") ; "abz2025_one_lane_three_cars.stark")]
#[test_case(include_str!("../../../examples/stark/abz2025_two_lanes_two_cars.stark") ; "abz2025_two_lanes_two_cars.stark")]
#[test_case(include_str!("../../../examples/stark/polistil_race.stark") ; "polistil_race.stark")]
#[test_case(include_str!("../../../examples/stark/ventilator.stark") ; "ventilator.stark")]
fn checks_example_specification(source: &str) {
    let spec = UntypedStarkSpecification::parse(source).unwrap_or_else(|e| panic!("failed to parse: {e}"));

    if let Err(diagnostics) = spec.check() {
        panic!("failed to check:\n{}", diagnostics.render(source));
    }
}
