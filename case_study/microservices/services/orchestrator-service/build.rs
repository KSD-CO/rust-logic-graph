fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Compile all proto files for all services
    tonic_build::compile_protos("../../proto/oms.proto")?;
    tonic_build::compile_protos("../../proto/inventory.proto")?;
    tonic_build::compile_protos("../../proto/supplier.proto")?;
    tonic_build::compile_protos("../../proto/uom.proto")?;
    tonic_build::compile_protos("../../proto/rule_engine.proto")?;
    tonic_build::compile_protos("../../proto/po.proto")?;
    Ok(())
}
