use criterion::{black_box, criterion_group, criterion_main, Criterion};
use ddex_builder::{DDEXBuilder, BuildRequest, BuildOptions};

fn benchmark_building(c: &mut Criterion) {
    c.bench_function("build_simple_release", |b| {
        let builder = DDEXBuilder::new();
        let request = create_test_request();
        
        b.iter(|| {
            let result = builder.build(
                black_box(request.clone()), 
                black_box(BuildOptions::default())
            );
            black_box(result)
        });
    });
}

fn create_test_request() -> BuildRequest {
    use ddex_builder::builder::{
        MessageHeaderRequest, PartyRequest, LocalizedStringRequest
    };
    
    BuildRequest {
        header: MessageHeaderRequest {
            message_id: Some("BENCH_001".to_string()),
            message_sender: PartyRequest {
                party_name: vec![LocalizedStringRequest {
                    text: "Benchmark Sender".to_string(),
                    language_code: None,
                }],
                party_id: None,
            },
            message_recipient: PartyRequest {
                party_name: vec![LocalizedStringRequest {
                    text: "Benchmark Recipient".to_string(),
                    language_code: None,
                }],
                party_id: None,
            },
            message_control_type: None,
        },
        version: "4.3".to_string(),
        profile: None,
        releases: vec![],
        deals: vec![],
        extensions: None,
    }
}

criterion_group!(benches, benchmark_building);
criterion_main!(benches);