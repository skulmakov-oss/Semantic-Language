use semantic_language::frontend::{emit_ir_to_semcode, IrFunction, IrInstr};
use semantic_language::prom_abi::AbiValue;
use semantic_language::prom_cap::CapabilityManifest;
use semantic_language::prom_gates::{DeterministicGateMock, GateDescriptor, GateId, GateRegistry};
use semantic_language::prom_runtime::GateExecutionSession;
use semantic_language::runtime_core::ExecutionContext;

fn runtime_program() -> Vec<IrFunction> {
    vec![IrFunction {
        name: "main".to_string(),
        instrs: vec![
            IrInstr::GateRead {
                dst: 0,
                device_id: 7,
                port: 3,
            },
            IrInstr::GateWrite {
                device_id: 7,
                port: 4,
                src: 0,
            },
            IrInstr::Ret { src: None },
        ],
        ownership_events: Vec::new(),
    }]
}

#[test]
fn gate_execution_session_runs_verified_program_with_bound_registry() {
    let bytes = emit_ir_to_semcode(&runtime_program(), false).expect("emit");

    let mut registry = GateRegistry::new();
    registry
        .register(GateDescriptor::read_only(7, 3, "sensor.alpha"))
        .expect("register read gate");
    registry
        .register(GateDescriptor::read_write(7, 4, "actuator.beta"))
        .expect("register write gate");

    let manifest = CapabilityManifest::gate_surface();
    let metadata = manifest.metadata();
    let mut binding = DeterministicGateMock::new();
    binding.seed_read(GateId::new(7, 3), AbiValue::I32(99));

    let mut session =
        GateExecutionSession::kernel_bound(&registry, &mut binding, &manifest, metadata.clone());
    assert_eq!(session.descriptor().context, ExecutionContext::KernelBound);
    assert!(session.descriptor().gate_registry_bound);
    assert_eq!(session.descriptor().capability_manifest, metadata);

    session
        .run_verified_semcode(&bytes)
        .expect("run verified via runtime session");

    drop(session);
    assert_eq!(binding.writes(), &[(GateId::new(7, 4), AbiValue::I32(99))]);
}
