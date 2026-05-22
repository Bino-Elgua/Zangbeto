use omo_diagnostic::agent::diagnostic_handler::{DiagnosticHandler, HandlerAction};
use omo_diagnostic::zangbeto_client::ZangbetoClient;
use tokio::time::{interval, Duration};
use tracing::{info, warn, error};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    info!("🌀 Starting Zàngbétò Native Steward");

    let (diag_tx, mut diag_rx) = tokio::sync::mpsc::channel(100);
    let zangbeto = ZangbetoClient::new("http://localhost:9000".into(), "0xguardian".into());
    
    let archive = omo_diagnostic::agent::archive::Archive::new(&std::env::current_dir()?);
    
    let handler = DiagnosticHandler::new(
        zangbeto,
        diag_tx,
        omo_diagnostic::Severity::Warning,
        std::env::current_dir()?,
    );

    let mut kernel_engine = omo_kernel::kernel::engine::StateTransitionEngine::new();
    let mut current_state = omo_kernel::kernel::css::CanonicalSystemState::default();
    let env_ctx = omo_kernel::kernel::engine::EnvironmentContext {
        timestamp: chrono::Utc::now().timestamp() as u64,
        external_signals: std::collections::HashMap::new(),
    };

    info!("🌐 Reality VM initialized: {}", current_state.state_hash);

    // ─────────────────────────────────────────────────────
    // Native Sui Event Listener (Mocked)
    // ─────────────────────────────────────────────────────
    let listener_handle = tokio::spawn(async move {
        let mut ticker = interval(Duration::from_secs(30));
        info!("📡 Native Event Listener active (polling mock)");
        
        loop {
            ticker.tick().await;
            // In a real implementation, we would use sui-sdk to subscribe to events
        }
    });

    // ─────────────────────────────────────────────────────
    // Diagnostic Processor Loop
    // ─────────────────────────────────────────────────────
    while let Some(diag) = diag_rx.recv().await {
        info!("📩 Processing diagnostic: {}", diag.diagnostic.code);
        let _ = archive.store_receipt(&diag);

        // Update Reality VM state
        let intent = format!("Handle diagnostic {}", diag.diagnostic.code);
        match kernel_engine.transition(current_state.clone(), intent, env_ctx.clone()).await {
            Ok(new_state) => {
                current_state = new_state;
                info!("🜂 Reality updated: {}", current_state.state_hash);
            }
            Err(e) => warn!("⚠️ Reality transition failed: {}", e),
        }
        
        if handler.should_auto_merge(&diag) {
            info!("🛠  Executing auto-repair for {}", diag.diagnostic.code);
            match handler.execute_repair(&diag).await {
                Ok(success) => {
                    if success {
                        info!("✅ Repair successful for {}", diag.diagnostic.code);
                    } else {
                        warn!("⚠️  Repair failed for {}", diag.diagnostic.code);
                    }
                }
                Err(e) => error!("❌ Error during repair: {}", e),
            }
        } else {
            info!("⚖️  Diagnostic requires manual review or high severity");
        }
    }

    let _ = archive.seal_week();

    let _ = listener_handle.await;
    Ok(())
}
