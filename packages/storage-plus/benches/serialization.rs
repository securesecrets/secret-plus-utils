use better_secret_math::U256;
use borsh::{BorshDeserialize, BorshSerialize};
use cosmwasm_std::testing::MockStorage;
use criterion::{black_box, criterion_group, Criterion};
use secret_borsh_storage::BorshItem;
use serde::{Deserialize, Serialize};

use secret_storage_plus::{Bincode2, Item, Json, Serde};

#[derive(Serialize, Deserialize, BorshDeserialize, BorshSerialize, PartialEq, Debug)]
struct Config {
    pub owner: String,
    pub address: String,
    pub max_tokens: i32,
}

impl Config {
    pub fn new() -> Self {
        Config {
            owner: "iaudnfuafbuwiafnawiuofngsgjknsjgnsogvnsgjnaigojanfasjknfakjnakfjanfkasfnafkjsafnajkfnajfanfka".to_string(),
            address: "iaudnfuafbuwiafnawiuofngsgjknsjgnsogvnsgjnaigojanfasjknfakjnakfjanfkasfnafkjsafnajkfnajfanfka".to_string(),
            max_tokens: i32::MAX,
        }
    }
}

#[derive(Serialize, Deserialize, BorshDeserialize, BorshSerialize, PartialEq, Debug)]
struct BigChungis {
    pub owner: String,
    pub address: String,
    pub b: [U256; 2],
    pub i: [U256; 2],
    pub g: [U256; 2],
    pub chungis: [U256; 10],
}

impl BigChungis {
    pub fn big() -> Self {
        BigChungis {
            owner: "iaudnfuafbuwiafnawiuofngsgjknsjgnsogvnsgjnaigojanfasjknfakjnakfjanfkasfnafkjsafnajkfnajfanfka".to_string(),
            address: "iaudnfuafbuwiafnawiuofngsgjknsjgnsogvnsgjnaigojanfasjknfakjnakfjanfkasfnafkjsafnajkfnajfanfka".to_string(),
            b: [U256::new(1029781591238083739u128); 2],
            i: [U256::MAX; 2],
            g: [U256::MAX; 2],
            chungis: [U256::MAX; 10],
        }
    }
}

const CONFIG: Item<Config, Bincode2> = Item::new("blahblahbalh");
const CONFIG_JSON: Item<Config, Json> = Item::new("blahblahbalh");
const CONFIG_BORSH: BorshItem<Config> = BorshItem::new("adwadwa");

const BIG: Item<BigChungis, Bincode2> = Item::new("blahblahbalh");
const BIG_JSON: Item<BigChungis, Json> = Item::new("blahblahbalh");
const BIG_BORSH: BorshItem<BigChungis> = BorshItem::new("adwadwa");

fn big_save_and_load(item: Item<BigChungis, impl Serde>) {
    let mut store = MockStorage::new();

    assert!(item.load(&store).is_err());
    assert_eq!(item.may_load(&store).unwrap(), None);

    let big = BigChungis::big();
    item.save(&mut store, &big).unwrap();

    assert_eq!(big, item.load(&store).unwrap());
}

fn borsh_big_save_and_load(item: BorshItem<BigChungis>) {
    let mut store = MockStorage::new();

    assert!(item.load(&store).is_err());
    assert_eq!(item.may_load(&store).unwrap(), None);

    let big = BigChungis::big();
    item.save(&mut store, &big).unwrap();

    assert_eq!(big, item.load(&store).unwrap());
}

fn borsh_save_and_load(item: BorshItem<Config>) {
    let mut store = MockStorage::new();

    assert!(item.load(&store).is_err());
    assert_eq!(item.may_load(&store).unwrap(), None);

    let cfg = Config::new();
    item.save(&mut store, &cfg).unwrap();

    assert_eq!(cfg, item.load(&store).unwrap());
}

fn save_and_load(item: Item<Config, impl Serde>) {
    let mut store = MockStorage::new();

    assert!(item.load(&store).is_err());
    assert_eq!(item.may_load(&store).unwrap(), None);

    let cfg = Config::new();
    item.save(&mut store, &cfg).unwrap();

    assert_eq!(cfg, item.load(&store).unwrap());
}

fn bench_bincode2_json(c: &mut Criterion) {
    let mut group = c.benchmark_group("Bincode vs Json");

    group.bench_function("bincode2 save & load", |b| {
        b.iter(|| save_and_load(black_box(CONFIG)))
    });

    group.bench_function("json save & load", |b| {
        b.iter(|| save_and_load(black_box(CONFIG_JSON)))
    });

    group.bench_function("borsh save & load", |b| {
        b.iter(|| borsh_save_and_load(black_box(CONFIG_BORSH)))
    });

    group.bench_function("bincode2 big save & load", |b| {
        b.iter(|| big_save_and_load(black_box(BIG)))
    });

    group.bench_function("json big save & load", |b| {
        b.iter(|| big_save_and_load(black_box(BIG_JSON)))
    });

    group.bench_function("borsh big save & load", |b| {
        b.iter(|| borsh_big_save_and_load(black_box(BIG_BORSH)))
    });

    group.finish();
}

criterion_group!(benches, bench_bincode2_json);
