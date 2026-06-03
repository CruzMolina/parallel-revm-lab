use std::error::Error;
use std::path::Path;

use parallel_revm_lab_analyzer::{
    analyze_trace_pack, dossier_schedule_trace_json, render_dossier_html, render_dossier_markdown,
};
use parallel_revm_lab_revm_smoke::{smoke_trace_pack, BytecodeFixture, RevmSmokeTx};
use revm::primitives::Address;

fn main() -> Result<(), Box<dyn Error>> {
    let contract = Address::new([0x22; 20]);
    let txs = vec![
        RevmSmokeTx {
            tx_index: 0,
            caller: Address::new([0x10; 20]),
            contract,
            fixture: BytecodeFixture::HotSlot { slot: 7 },
        },
        RevmSmokeTx {
            tx_index: 1,
            caller: Address::new([0x11; 20]),
            contract,
            fixture: BytecodeFixture::WriteSlot { slot: 8 },
        },
        RevmSmokeTx {
            tx_index: 2,
            caller: Address::new([0x12; 20]),
            contract,
            fixture: BytecodeFixture::HotSlot { slot: 7 },
        },
    ];
    let pack = smoke_trace_pack(&txs)?;
    let trace_dir = Path::new("trace-packs/revm-smoke-mini");
    pack.write_dir(trace_dir)?;

    let dossier = analyze_trace_pack(&pack, &[1, 2, 4]);
    std::fs::create_dir_all("reports")?;
    serde_json::to_writer_pretty(
        std::fs::File::create("reports/revm-smoke-dossier.json")?,
        &dossier,
    )?;
    std::fs::write(
        "reports/revm-smoke-dossier.md",
        render_dossier_markdown(&dossier),
    )?;
    std::fs::write(
        "reports/revm-smoke-dossier.html",
        render_dossier_html(
            &dossier,
            "cargo run -p parallel-revm-lab-revm-smoke --example emit_trace_pack",
        ),
    )?;
    serde_json::to_writer_pretty(
        std::fs::File::create("reports/revm-smoke-schedule.trace.json")?,
        &dossier_schedule_trace_json(&dossier),
    )?;

    println!(
        "wrote {} and reports/revm-smoke-dossier.json",
        trace_dir.display()
    );
    Ok(())
}
