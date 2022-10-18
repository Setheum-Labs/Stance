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

#[cfg(feature = "try-runtime")]
use frame_support::ensure;
use frame_support::{
    log, storage_alias,
    traits::{Get, OnRuntimeUpgrade, PalletInfoAccess, StorageVersion},
    weights::Weight,
};
#[cfg(feature = "try-runtime")]
use stance_support::ensure_storage_version;
use stance_support::StorageMigration;
use primitives::SessionIndex;
use sp_std::vec::Vec;

use crate::Config;

type Accounts<T> = Vec<<T as frame_system::Config>::AccountId>;

#[storage_alias]
type SessionForValidatorsChange = StorageValue<StanceFGCompanion, SessionIndex>;

#[storage_alias]
type Validators<T> = StorageValue<StanceFGCompanion, Accounts<T>>;

/// Flattening double `Option<>` storage.
pub struct Migration<T, P>(sp_std::marker::PhantomData<(T, P)>);

impl<T: Config, P: PalletInfoAccess> StorageMigration for Migration<T, P> {
    #[cfg(feature = "try-runtime")]
    const MIGRATION_STORAGE_PREFIX: &'static [u8] = b"STANCE_FINALITY_COMPANION::V0_TO_V1_MIGRATION";
}

impl<T: Config, P: PalletInfoAccess> OnRuntimeUpgrade for Migration<T, P> {
    fn on_runtime_upgrade() -> Weight {
        log::info!(target: "stance_finality_companion", "Running migration from STORAGE_VERSION 0 to 1");

        let mut writes = 0;

        match SessionForValidatorsChange::translate(
            |old: Option<Option<SessionIndex>>| -> Option<SessionIndex> {
                log::info!(target: "stance_finality_companion", "Current storage value for SessionForValidatorsChange {:?}", old);
                match old {
                    Some(Some(x)) => Some(x),
                    _ => None,
                }
            },
        ) {
            Ok(_) => {
                writes += 1;
                log::info!(target: "stance_finality_companion", "Successfully migrated storage for SessionForValidatorsChange");
            }
            Err(why) => {
                log::error!(target: "stance_finality_companion", "Something went wrong during the migration of SessionForValidatorsChange {:?}", why);
            }
        };

        match Validators::<T>::translate(
            |old: Option<Option<Vec<T::AccountId>>>| -> Option<Vec<T::AccountId>> {
                log::info!(target: "stance_finality_companion", "Current storage value for Validators {:?}", old);
                match old {
                    Some(Some(x)) => Some(x),
                    _ => None,
                }
            },
        ) {
            Ok(_) => {
                writes += 1;
                log::info!(target: "stance_finality_companion", "Successfully migrated storage for Validators");
            }
            Err(why) => {
                log::error!(target: "stance_finality_companion", "Something went wrong during the migration of Validators storage {:?}", why);
            }
        };

        // store new version
        StorageVersion::new(1).put::<P>();
        writes += 1;

        T::DbWeight::get().reads(2) + T::DbWeight::get().writes(writes)
    }

    #[cfg(feature = "try-runtime")]
    fn pre_upgrade() -> Result<(), &'static str> {
        #[storage_alias]
        type SessionForValidatorsChange = StorageValue<StanceFGCompanion, Option<SessionIndex>>;
        #[storage_alias]
        type Validators<T> = StorageValue<StanceFGCompanion, Option<Accounts<T>>>;

        ensure_storage_version::<P>(0)?;

        Self::store_temp("session", SessionForValidatorsChange::get());
        Self::store_temp("validators", Validators::<T>::get());

        Ok(())
    }

    #[cfg(feature = "try-runtime")]
    fn post_upgrade() -> Result<(), &'static str> {
        ensure_storage_version::<P>(1)?;

        let new_session = SessionForValidatorsChange::get();
        let old_session = Self::read_temp::<Option<Option<SessionIndex>>>("session");

        match old_session {
            Some(Some(session)) => ensure!(
                Some(session) == new_session,
                "Mismatch on `SessionForValidatorsChange`",
            ),
            _ => ensure!(
                None == new_session,
                "New `SessionForValidatorsChange` should be `None`"
            ),
        };

        let new_validators = Validators::<T>::get();
        let old_validators = Self::read_temp::<Option<Option<Accounts<T>>>>("validators");

        match old_validators {
            Some(Some(validators)) => ensure!(
                Some(validators) == new_validators,
                "Mismatch on `Validators`",
            ),
            _ => ensure!(None == new_validators, "New `Validators` should be `None`"),
        };

        Ok(())
    }
}
