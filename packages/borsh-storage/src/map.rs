use crate::keys::{Key, PrimaryKey};
use crate::path::Path;
use borsh::{BorshDeserialize, BorshSerialize};
use cosmwasm_std::{StdError, StdResult, Storage};
use std::marker::PhantomData;

#[derive(Debug, Clone)]
pub struct Map<'a, K, T: BorshSerialize + BorshDeserialize> {
    namespace: &'a [u8],
    // see https://doc.rust-lang.org/std/marker/struct.PhantomData.html#unused-type-parameters for why this is needed
    key_type: PhantomData<K>,
    data_type: PhantomData<T>,
}

impl<'a, K, T: BorshSerialize + BorshDeserialize> Map<'a, K, T> {
    pub const fn new(namespace: &'a str) -> Self {
        Map {
            namespace: namespace.as_bytes(),
            data_type: PhantomData,
            key_type: PhantomData,
        }
    }

    pub fn namespace(&self) -> &'a [u8] {
        self.namespace
    }
}

impl<'a, K, T: BorshSerialize + BorshDeserialize> Map<'a, K, T>
where
    K: PrimaryKey<'a>,
{
    pub fn key(&self, k: K) -> Path<T> {
        Path::new(
            self.namespace,
            &k.key().iter().map(Key::as_ref).collect::<Vec<_>>(),
        )
    }

    pub fn save(&self, store: &mut dyn Storage, k: K, data: &T) -> StdResult<()> {
        self.key(k).save(store, data)
    }

    pub fn remove(&self, store: &mut dyn Storage, k: K) {
        self.key(k).remove(store)
    }

    /// load will return an error if no data is set at the given key, or on parse error
    pub fn load(&self, store: &dyn Storage, k: K) -> StdResult<T> {
        self.key(k).load(store)
    }

    /// may_load will parse the data stored at the key if present, returns Ok(None) if no data there.
    /// returns an error on issues parsing
    pub fn may_load(&self, store: &dyn Storage, k: K) -> StdResult<Option<T>> {
        self.key(k).may_load(store)
    }

    /// has returns true or false if any data is at this key, without parsing or interpreting the
    /// contents.
    pub fn has(&self, store: &dyn Storage, k: K) -> bool {
        self.key(k).has(store)
    }

    /// Loads the data, perform the specified action, and store the result
    /// in the database. This is shorthand for some common sequences, which may be useful.
    ///
    /// If the data exists, `action(Some(value))` is called. Otherwise `action(None)` is called.
    pub fn update<A, E>(&self, store: &mut dyn Storage, k: K, action: A) -> Result<T, E>
    where
        A: FnOnce(Option<T>) -> Result<T, E>,
        E: From<StdError>,
    {
        self.key(k).update(store, action)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use rstest::*;
    use rstest_reuse::{self, *};
    use std::ops::Deref;

    use crate::int_key::CwIntKey;
    use cosmwasm_std::testing::MockStorage;

    #[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone)]
    struct Data {
        pub name: String,
        pub age: i32,
    }

    const PEOPLE: Map<&[u8], Data> = Map::new("people");
    const ALLOWANCE: Map<(&[u8], &[u8]), u64> = Map::new("allow");
    const TRIPLE: Map<(&[u8], u8, &str), u64> = Map::new("triple");

    type Key<'a> = &'a [u8];
    type AKey<'a> = (&'a [u8], &'a [u8]);
    type TKey<'a> = (&'a [u8], u8, &'a str);

    #[template]
    #[rstest]
    #[case(PEOPLE, ALLOWANCE, TRIPLE)]
    fn serialization_3(
        #[case] people: Map<Key, Data>,
        #[case] allowance: Map<AKey, u64>,
        #[case] triple: Map<TKey, u64>,
    ) {
    }

    #[apply(serialization_3)]
    fn create_path(
        #[case] people: Map<Key, Data>,
        #[case] allowance: Map<AKey, u64>,
        #[case] triple: Map<TKey, u64>,
    ) {
        let path = people.key(b"john");
        let key = path.deref();
        // this should be prefixed(people) || john
        assert_eq!("people".len() + "john".len() + 2, key.len());
        assert_eq!(b"people".to_vec().as_slice(), &key[2..8]);
        assert_eq!(b"john".to_vec().as_slice(), &key[8..]);

        let path = allowance.key((b"john", b"maria"));
        let key = path.deref();
        // this should be prefixed(allow) || prefixed(john) || maria
        assert_eq!(
            "allow".len() + "john".len() + "maria".len() + 2 * 2,
            key.len()
        );
        assert_eq!(b"allow".to_vec().as_slice(), &key[2..7]);
        assert_eq!(b"john".to_vec().as_slice(), &key[9..13]);
        assert_eq!(b"maria".to_vec().as_slice(), &key[13..]);

        let path = triple.key((b"john", 8u8, "pedro"));
        let key = path.deref();
        // this should be prefixed(allow) || prefixed(john) || maria
        assert_eq!(
            "triple".len() + "john".len() + 1 + "pedro".len() + 2 * 3,
            key.len()
        );
        assert_eq!(b"triple".to_vec().as_slice(), &key[2..8]);
        assert_eq!(b"john".to_vec().as_slice(), &key[10..14]);
        assert_eq!(8u8.to_cw_bytes(), &key[16..17]);
        assert_eq!(b"pedro".to_vec().as_slice(), &key[17..]);
    }

    #[rstest]
    fn save_and_load(#[values(PEOPLE)] people: Map<Key, Data>) {
        let mut store = MockStorage::new();

        // save and load on one key
        let john = people.key(b"john");
        let data = Data {
            name: "John".to_string(),
            age: 32,
        };
        assert_eq!(None, john.may_load(&store).unwrap());
        john.save(&mut store, &data).unwrap();
        assert_eq!(data, john.load(&store).unwrap());

        // nothing on another key
        assert_eq!(None, people.may_load(&store, b"jack").unwrap());

        // same named path gets the data
        assert_eq!(data, people.load(&store, b"john").unwrap());

        // removing leaves us empty
        john.remove(&mut store);
        assert_eq!(None, john.may_load(&store).unwrap());
    }

    #[rstest]
    fn existence(#[values(PEOPLE)] people: Map<Key, Data>) {
        let mut store = MockStorage::new();

        // set data in proper format
        let data = Data {
            name: "John".to_string(),
            age: 32,
        };
        people.save(&mut store, b"john", &data).unwrap();

        // set and remove it
        people.save(&mut store, b"removed", &data).unwrap();
        people.remove(&mut store, b"removed");

        // invalid, but non-empty data
        store.set(&people.key(b"random"), b"random-data");

        // any data, including invalid or empty is returned as "has"
        assert!(people.has(&store, b"john"));
        assert!(people.has(&store, b"random"));

        // if nothing was written, it is false
        assert!(!people.has(&store, b"never-writen"));
        assert!(!people.has(&store, b"removed"));
    }

    #[rstest]
    fn composite_keys(#[values(ALLOWANCE)] allowance: Map<AKey, u64>) {
        let mut store = MockStorage::new();

        // save and load on a composite key
        let allow = allowance.key((b"owner", b"spender"));
        assert_eq!(None, allow.may_load(&store).unwrap());
        allow.save(&mut store, &1234).unwrap();
        assert_eq!(1234, allow.load(&store).unwrap());

        // not under other key
        let different = allowance.may_load(&store, (b"owners", b"pender")).unwrap();
        assert_eq!(None, different);

        // matches under a proper copy
        let same = allowance.load(&store, (b"owner", b"spender")).unwrap();
        assert_eq!(1234, same);
    }

    #[rstest]
    fn triple_keys(#[values(TRIPLE)] triple: Map<TKey, u64>) {
        let mut store = MockStorage::new();

        // save and load on a triple composite key
        let triple_path = triple.key((b"owner", 10u8, "recipient"));
        assert_eq!(None, triple_path.may_load(&store).unwrap());
        triple_path.save(&mut store, &1234).unwrap();
        assert_eq!(1234, triple_path.load(&store).unwrap());

        // not under other key
        let different = TRIPLE
            .may_load(&store, (b"owners", 10u8, "recipient"))
            .unwrap();
        assert_eq!(None, different);

        // matches under a proper copy
        let same = triple.load(&store, (b"owner", 10u8, "recipient")).unwrap();
        assert_eq!(1234, same);
    }

    #[rstest]
    fn basic_update(#[values(ALLOWANCE)] allowance: Map<AKey, u64>) {
        let mut store = MockStorage::new();

        let add_ten = |a: Option<u64>| -> StdResult<_> { Ok(a.unwrap_or_default() + 10) };

        // save and load on three keys, one under different owner
        let key: (&[u8], &[u8]) = (b"owner", b"spender");
        allowance.update(&mut store, key, add_ten).unwrap();
        let twenty = allowance.update(&mut store, key, add_ten).unwrap();
        assert_eq!(20, twenty);
        let loaded = allowance.load(&store, key).unwrap();
        assert_eq!(20, loaded);
    }

    #[rstest]
    fn readme_works(#[values(PEOPLE)] people: Map<Key, Data>) -> StdResult<()> {
        let mut store = MockStorage::new();
        let data = Data {
            name: "John".to_string(),
            age: 32,
        };

        // load and save with extra key argument
        let empty = people.may_load(&store, b"john")?;
        assert_eq!(None, empty);
        people.save(&mut store, b"john", &data)?;
        let loaded = people.load(&store, b"john")?;
        assert_eq!(data, loaded);

        // nothing on another key
        let missing = people.may_load(&store, b"jack")?;
        assert_eq!(None, missing);

        // update function for new or existing keys
        let birthday = |d: Option<Data>| -> StdResult<Data> {
            match d {
                Some(one) => Ok(Data {
                    name: one.name,
                    age: one.age + 1,
                }),
                None => Ok(Data {
                    name: "Newborn".to_string(),
                    age: 0,
                }),
            }
        };

        let old_john = people.update(&mut store, b"john", birthday)?;
        assert_eq!(33, old_john.age);
        assert_eq!("John", old_john.name.as_str());

        let new_jack = people.update(&mut store, b"jack", birthday)?;
        assert_eq!(0, new_jack.age);
        assert_eq!("Newborn", new_jack.name.as_str());

        // update also changes the store
        assert_eq!(old_john, people.load(&store, b"john")?);
        assert_eq!(new_jack, people.load(&store, b"jack")?);

        // removing leaves us empty
        people.remove(&mut store, b"john");
        let empty = people.may_load(&store, b"john")?;
        assert_eq!(None, empty);

        Ok(())
    }

    #[rstest]
    fn readme_works_composite_keys(
        #[values(ALLOWANCE)] allowance: Map<AKey, u64>,
    ) -> StdResult<()> {
        let mut store = MockStorage::new();

        // save and load on a composite key
        let empty = allowance.may_load(&store, (b"owner", b"spender"))?;
        assert_eq!(None, empty);
        allowance.save(&mut store, (b"owner", b"spender"), &777)?;
        let loaded = allowance.load(&store, (b"owner", b"spender"))?;
        assert_eq!(777, loaded);

        // doesn't appear under other key (even if a concat would be the same)
        let different = allowance.may_load(&store, (b"owners", b"pender")).unwrap();
        assert_eq!(None, different);

        // simple update
        allowance.update(&mut store, (b"owner", b"spender"), |v| -> StdResult<u64> {
            Ok(v.unwrap_or_default() + 222)
        })?;
        let loaded = allowance.load(&store, (b"owner", b"spender"))?;
        assert_eq!(999, loaded);

        Ok(())
    }

    #[rstest]
    fn readme_works_with_path(
        #[values(PEOPLE)] people: Map<Key, Data>,
        #[values(ALLOWANCE)] allowance: Map<AKey, u64>,
    ) -> StdResult<()> {
        let mut store = MockStorage::new();
        let data = Data {
            name: "John".to_string(),
            age: 32,
        };

        // create a Path one time to use below
        let john = people.key(b"john");

        // Use this just like an Item above
        let empty = john.may_load(&store)?;
        assert_eq!(None, empty);
        john.save(&mut store, &data)?;
        let loaded = john.load(&store)?;
        assert_eq!(data, loaded);
        john.remove(&mut store);
        let empty = john.may_load(&store)?;
        assert_eq!(None, empty);

        // same for composite keys, just use both parts in key()
        let allow = allowance.key((b"owner", b"spender"));
        allow.save(&mut store, &1234)?;
        let loaded = allow.load(&store)?;
        assert_eq!(1234, loaded);
        allow.update(&mut store, |x| -> StdResult<u64> {
            Ok(x.unwrap_or_default() * 2)
        })?;
        let loaded = allow.load(&store)?;
        assert_eq!(2468, loaded);

        Ok(())
    }
}
