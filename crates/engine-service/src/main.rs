//! engine-service: a stateless gRPC rules engine wrapping `poker-engine`.
//!
//! M2 scaffold: this proves the proto contract generates and compiles on the
//! Rust side. The actual `EngineService` RPC implementations land in M3.

pub mod pokerpb {
    tonic::include_proto!("poker.v1");
}

fn main() {
    // Touch a generated type so codegen is exercised at compile time.
    let _sample = pokerpb::Action {
        kind: pokerpb::ActionKind::Fold as i32,
        amount: 0,
    };
    println!("engine-service scaffold built; proto codegen OK (M3 implements the RPCs)");
}
