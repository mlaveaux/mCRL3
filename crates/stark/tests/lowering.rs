//! Lowers every `.stark` file under `examples/stark/` end-to-end (parse ->
//! check -> [lower]) and asserts the resulting [IrProgram] is internally
//! consistent. Mirrors `tests/examples.rs`'s `checks_example_specification`
//! (same file list, one step further down the pipeline) — this is the
//! `lowers_every_example_specification` test `IR_LOWERING_PLAN.md` calls
//! for. Every example lowers, including the ones using perturbations,
//! distances and formulas.

use merc_stark::UntypedStarkSpecification;
use merc_stark::lower;
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
fn lowers_every_example_specification(source: &str) {
    let spec = UntypedStarkSpecification::parse(source)
        .unwrap_or_else(|e| panic!("failed to parse: {e}"))
        .check()
        .unwrap_or_else(|d| panic!("failed to check:\n{}", d.render(source)));
    let program = lower(&spec).unwrap_or_else(|d| panic!("failed to lower:\n{}", d.render(source)));

    program
        .validate()
        .unwrap_or_else(|e| panic!("lowered an inconsistent arena: {e}"));
}
