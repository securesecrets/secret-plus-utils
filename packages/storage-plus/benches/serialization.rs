use cosmwasm_std::{testing::MockStorage, Uint512};
use criterion::{black_box, criterion_group, criterion_main, Criterion};

use rand::Rng;
use serde::{Deserialize, Serialize};
use std::mem;
use std::time::Duration;

use secret_storage_plus::{Item, Json, Bincode2, Serde};

#[derive(Serialize, Deserialize, PartialEq, Debug)]
struct Config {
    pub owner: String,
    pub max_tokens: i32,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
struct BigChungis {
    pub b: Uint512,
    pub i: Uint512,
    pub g: Uint512,
    pub chungis: Vec<Uint512>,
}

impl BigChungis {
    pub fn big() -> Self {
        BigChungis { b: Uint512::MAX, i: Uint512::MAX, g: Uint512::MAX, chungis: [Uint512::MAX; 10].to_vec() }
    }
}

const CONFIG: Item<Config, Bincode2> = Item::new("blahblahbalh");
const CONFIG_JSON: Item<Config, Json> = Item::new("blahblahbalh");

const BIG: Item<BigChungis, Bincode2> = Item::new("blahblahbalh");
const BIG_JSON: Item<BigChungis, Json> = Item::new("blahblahbalh");

fn big_save_and_load(item: Item<BigChungis, impl Serde>) {
    let mut store = MockStorage::new();

    assert!(item.load(&store).is_err());
    assert_eq!(item.may_load(&store).unwrap(), None);

    let big = BigChungis::big();
    item.save(&mut store, &big).unwrap();

    assert_eq!(big, item.load(&store).unwrap());
}

fn save_and_load(item: Item<Config, impl Serde>) {
    let mut store = MockStorage::new();

    assert!(item.load(&store).is_err());
    assert_eq!(item.may_load(&store).unwrap(), None);

    let cfg = Config {
        owner: "admin".to_string(),
        max_tokens: 1234,
    };
    item.save(&mut store, &cfg).unwrap();

    assert_eq!(cfg, item.load(&store).unwrap());
}

fn bench_bincode2_json(c: &mut Criterion) {
    let mut group = c.benchmark_group("Bincode vs Json");

    group.bench_function("bincode2 save & load", |b| {
        b.iter(|| {
            save_and_load(black_box(CONFIG))
        } )
    });

    group.bench_function("json save & load", |b| {
        b.iter(|| {
            save_and_load(black_box(CONFIG_JSON))
        } )
    });

    group.bench_function("bincode2 big save & load", |b| {
        b.iter(|| {
            big_save_and_load(black_box(BIG))
        } )
    });

    group.bench_function("json big save & load", |b| {
        b.iter(|| {
            big_save_and_load(black_box(BIG_JSON))
        } )
    });

    group.finish();
}

criterion_group!(benches, bench_bincode2_json);
