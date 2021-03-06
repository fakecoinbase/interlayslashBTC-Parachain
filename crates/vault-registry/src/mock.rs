/// Mocking the test environment
use frame_support::{
    impl_outer_event, impl_outer_origin, parameter_types,
    weights::{
        constants::{BlockExecutionWeight, ExtrinsicBaseWeight, RocksDbWeight},
        Weight,
    },
};
use mocktopus::mocking::clear_mocks;
use sp_core::H256;
use sp_runtime::{
    testing::Header,
    traits::{BlakeTwo256, IdentityLookup},
    Perbill,
};

use mocktopus::mocking::{MockResult, Mockable};

use crate::ext;
use crate::{GenesisConfig, Module, Trait};

impl_outer_origin! {
    pub enum Origin for Test {}
}

mod test_events {
    pub use crate::Event;
}

impl_outer_event! {
    pub enum TestEvent for Test {
        system<T>,
        test_events<T>,
        pallet_balances<T>,
        collateral<T>,
        treasury<T>,
        exchange_rate_oracle<T>,
        security,
    }
}

pub type AccountId = u64;
pub type Balance = u64;
pub type BlockNumber = u64;

// For testing the pallet, we construct most of a mock runtime. This means
// first constructing a configuration type (`Test`) which `impl`s each of the
// configuration traits of modules we want to use.
#[derive(Clone, Eq, PartialEq)]
pub struct Test;

parameter_types! {
    pub const BlockHashCount: u64 = 250;
    pub const MaximumBlockWeight: Weight = 1024;
    pub const MaximumBlockLength: u32 = 2 * 1024;
    pub const AvailableBlockRatio: Perbill = Perbill::from_percent(75);
}

impl system::Trait for Test {
    type AccountId = AccountId;
    type Call = ();
    type Lookup = IdentityLookup<Self::AccountId>;
    type Index = u64;
    type BlockNumber = BlockNumber;
    type Hash = H256;
    type Hashing = BlakeTwo256;
    type Header = Header;
    type Event = TestEvent;
    type Origin = Origin;
    type BlockHashCount = BlockHashCount;
    type MaximumBlockWeight = MaximumBlockWeight;
    type MaximumBlockLength = MaximumBlockLength;
    type AvailableBlockRatio = AvailableBlockRatio;
    type BlockExecutionWeight = BlockExecutionWeight;
    type DbWeight = RocksDbWeight;
    type ExtrinsicBaseWeight = ExtrinsicBaseWeight;
    type Version = ();
    type ModuleToIndex = ();
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type AccountData = pallet_balances::AccountData<u64>;
}

parameter_types! {
    pub const ExistentialDeposit: u64 = 1;
}

impl pallet_balances::Trait for Test {
    type Balance = Balance;
    type Event = TestEvent;
    type DustRemoval = ();
    type ExistentialDeposit = ExistentialDeposit;
    type AccountStore = System;
}

impl collateral::Trait for Test {
    type DOT = Balances;
    type Event = TestEvent;
}

impl treasury::Trait for Test {
    type PolkaBTC = Balances;
    type Event = TestEvent;
}

parameter_types! {
    pub const MinimumPeriod: u64 = 5;
}
impl timestamp::Trait for Test {
    type Moment = u64;
    type OnTimestampSet = ();
    type MinimumPeriod = MinimumPeriod;
}

impl exchange_rate_oracle::Trait for Test {
    type Event = TestEvent;
}

impl Trait for Test {
    type Event = TestEvent;
}

impl security::Trait for Test {
    type Event = TestEvent;
}

pub type Balances = pallet_balances::Module<Test>;

// pub type Error = crate::Error;

pub type System = system::Module<Test>;
pub type VaultRegistry = Module<Test>;

pub struct ExtBuilder;

pub const DEFAULT_ID: u64 = 3;
pub const OTHER_ID: u64 = 4;
pub const RICH_ID: u64 = 5;
pub const DEFAULT_COLLATERAL: u64 = 100;
pub const RICH_COLLATERAL: u64 = DEFAULT_COLLATERAL + 50;

impl ExtBuilder {
    pub fn build() -> sp_io::TestExternalities {
        let mut storage = system::GenesisConfig::default()
            .build_storage::<Test>()
            .unwrap();

        pallet_balances::GenesisConfig::<Test> {
            balances: vec![
                (DEFAULT_ID, DEFAULT_COLLATERAL),
                (OTHER_ID, DEFAULT_COLLATERAL),
                (RICH_ID, RICH_COLLATERAL),
            ],
        }
        .assimilate_storage(&mut storage)
        .unwrap();

        GenesisConfig {
            secure_collateral_threshold: 100000,
        }
        .assimilate_storage(&mut storage)
        .unwrap();

        sp_io::TestExternalities::from(storage)
    }
}

pub fn run_test<T, U>(test: T) -> U
where
    T: FnOnce() -> U,
{
    clear_mocks();
    ext::oracle::dots_to_btc::<Test>.mock_safe(|v| MockResult::Return(Ok(v)));
    ext::oracle::btc_to_dots::<Test>.mock_safe(|v| MockResult::Return(Ok(v)));
    ExtBuilder::build().execute_with(|| {
        System::set_block_number(1);
        test()
    })
}
