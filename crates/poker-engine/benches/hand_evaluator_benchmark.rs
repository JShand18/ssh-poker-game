use criterion::{criterion_group, criterion_main, Criterion};
use poker_engine::{card::Card, deck::Deck, PokerCard, PokerEvaluator};
use rand::seq::SliceRandom;
use rand::thread_rng;

const NUM_HANDS_TO_EVALUATE: usize = 1000;

fn benchmark_hand_evaluation(c: &mut Criterion) {
    let mut cards: Vec<Card> = Deck::new().as_ref().to_vec();
    let mut rng = thread_rng();
    let mut hands_to_evaluate: Vec<Vec<PokerCard>> = Vec::with_capacity(NUM_HANDS_TO_EVALUATE);
    let evaluator = PokerEvaluator::new();

    for _ in 0..NUM_HANDS_TO_EVALUATE {
        cards.shuffle(&mut rng);
        let poker_cards: Vec<PokerCard> = cards[..5].iter().map(|c| c.into()).collect();
        hands_to_evaluate.push(poker_cards);
    }

    let mut i = 0;
    c.bench_function("hand_evaluation", |b| {
        b.iter(|| {
            let hand = evaluator.evaluate(&hands_to_evaluate[i]).unwrap();
            criterion::black_box(hand);
            i = (i + 1) % NUM_HANDS_TO_EVALUATE;
        })
    });
}

criterion_group!(benches, benchmark_hand_evaluation);
criterion_main!(benches); 