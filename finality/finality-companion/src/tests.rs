// بِسْمِ اللَّهِ الرَّحْمَنِ الرَّحِيم

// This file is part of STANCE.

// Copyright (C) 2019-Present Setheum Labs.
// SPDX-License-Identifier: GPL-3.0-or-later WITH Classpath-exception-2.0

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.

#![cfg(test)]

use frame_support::{storage_alias, traits::OneSessionHandler};

use crate::mock::*;

#[storage_alias]
type SessionForValidatorsChange = StorageValue<StanceFGCompanion, u32>;

#[storage_alias]
type Validators<T> = StorageValue<StanceFGCompanion, Vec<<T as frame_system::Config>::AccountId>>;

#[cfg(feature = "try-runtime")]
mod migration_tests {
    use frame_support::{storage::migration::put_storage_value, traits::StorageVersion};
    use stance_support::StorageMigration;

    use crate::{migrations, mock::*, Pallet};

    const MODULE: &[u8] = b"StanceFGCompanion";

    #[test]
    fn migration_from_v0_to_v1_works() {
        new_test_ext(&[(1u64, 1u64), (2u64, 2u64)]).execute_with(|| {
            StorageVersion::new(0).put::<Pallet<Test>>();

            put_storage_value(MODULE, b"SessionForValidatorsChange", &[], Some(7u32));
            put_storage_value(MODULE, b"Validators", &[], Some(vec![0u64, 1u64]));

            let _weight = migrations::v0_to_v1::Migration::<Test, StanceFGCompanion>::migrate();
        })
    }

    #[test]
    fn migration_from_v1_to_v2_works() {
        new_test_ext(&[(1u64, 1u64), (2u64, 2u64)]).execute_with(|| {
            StorageVersion::new(1).put::<Pallet<Test>>();

            put_storage_value(MODULE, b"SessionForValidatorsChange", &[], ());
            put_storage_value(MODULE, b"Validators", &[], ());
            put_storage_value(MODULE, b"MillisecsPerBlock", &[], ());
            put_storage_value(MODULE, b"SessionPeriod", &[], ());

            let _weight = migrations::v1_to_v2::Migration::<Test, StanceFGCompanion>::migrate();
        })
    }
}

#[test]
fn test_update_authorities() {
    new_test_ext(&[(1u64, 1u64), (2u64, 2u64)]).execute_with(|| {
        initialize_session();
        run_session(1);

        StanceFGCompanion::update_authorities(to_authorities(&[2, 3, 4]).as_slice());

        assert_eq!(StanceFGCompanion::authorities(), to_authorities(&[2, 3, 4]));
    });
}

#[test]
fn test_initialize_authorities() {
    new_test_ext(&[(1u64, 1u64), (2u64, 2u64)]).execute_with(|| {
        assert_eq!(StanceFGCompanion::authorities(), to_authorities(&[1, 2]));
    });
}

#[test]
#[should_panic]
fn fails_to_initialize_again_authorities() {
    new_test_ext(&[(1u64, 1u64), (2u64, 2u64)]).execute_with(|| {
        StanceFGCompanion::initialize_authorities(&to_authorities(&[1, 2, 3]));
    });
}

#[test]
fn test_current_authorities() {
    new_test_ext(&[(1u64, 1u64), (2u64, 2u64)]).execute_with(|| {
        initialize_session();

        run_session(1);

        StanceFGCompanion::update_authorities(to_authorities(&[2, 3, 4]).as_slice());

        assert_eq!(StanceFGCompanion::authorities(), to_authorities(&[2, 3, 4]));

        run_session(2);

        StanceFGCompanion::update_authorities(to_authorities(&[1, 2, 3]).as_slice());

        assert_eq!(StanceFGCompanion::authorities(), to_authorities(&[1, 2, 3]));
    })
}

#[test]
fn test_session_rotation() {
    new_test_ext(&[(1u64, 1u64), (2u64, 2u64)]).execute_with(|| {
        initialize_session();
        run_session(1);

        let new_validators = new_session_validators(&[3u64, 4u64]);
        let queued_validators = new_session_validators(&[]);
        StanceFGCompanion::on_new_session(true, new_validators, queued_validators);
        assert_eq!(StanceFGCompanion::authorities(), to_authorities(&[3, 4]));
    })
}

#[test]
fn test_emergency_signer() {
    new_test_ext(&[(1u64, 1u64), (2u64, 2u64)]).execute_with(|| {
        initialize_session();

        run_session(1);

        StanceFGCompanion::set_next_emergency_finalizer(to_authority(&21));

        assert_eq!(StanceFGCompanion::emergency_finalizer(), None);
        assert_eq!(StanceFGCompanion::queued_emergency_finalizer(), None);

        run_session(2);

        StanceFGCompanion::set_next_emergency_finalizer(to_authority(&37));

        assert_eq!(StanceFGCompanion::emergency_finalizer(), None);
        assert_eq!(StanceFGCompanion::queued_emergency_finalizer(), Some(to_authority(&21)));

        run_session(3);

        assert_eq!(StanceFGCompanion::emergency_finalizer(), Some(to_authority(&21)));
        assert_eq!(StanceFGCompanion::queued_emergency_finalizer(), Some(to_authority(&37)));
    })
}
