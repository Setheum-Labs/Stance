#![allow(clippy::too_many_arguments, clippy::unnecessary_mut_passed)]
#![cfg_attr(not(feature = "std"), no_std)]
use codec::{Decode, Encode};
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use sp_runtime::ConsensusEngineId;
use sp_std::vec::Vec;

#[derive(Encode, Decode, PartialEq, Eq, Debug)]
pub enum ApiError {
    DecodeKey,
}

use sp_core::crypto::KeyTypeId;
pub const KEY_TYPE: KeyTypeId = KeyTypeId(*b"alp0");

mod app {
    use sp_application_crypto::{app_crypto, ed25519};
    app_crypto!(ed25519, crate::KEY_TYPE);
}

sp_application_crypto::with_pair! {
    pub type AuthorityPair = app::Pair;
}
pub type AuthorityId = app::Public;

pub type AuthoritySignature = app::Signature;

// Same as GRANDPA_ENGINE_ID because as of right now substrate sends only
// grandpa justifications over the network.
// TODO: change this once https://github.com/paritytech/substrate/issues/8172 will be resolved.
pub const STANCE_ENGINE_ID: ConsensusEngineId = *b"FRNK";


/// All the data needed to verify block finalization justifications.
#[derive(Clone, Debug, Encode, Decode, PartialEq, Eq)]
pub struct SessionAuthorityData {
    authorities: Vec<AuthorityId>,
    emergency_finalizer: Option<AuthorityId>,
}

impl SessionAuthorityData {
    pub fn new(authorities: Vec<AuthorityId>, emergency_finalizer: Option<AuthorityId>) -> Self {
        SessionAuthorityData {
            authorities,
            emergency_finalizer,
        }
    }

    pub fn authorities(&self) -> &Vec<AuthorityId> {
        &self.authorities
    }

    pub fn emergency_finalizer(&self) -> &Option<AuthorityId> {
        &self.emergency_finalizer
    }
}

sp_api::decl_runtime_apis! {
    pub trait StanceSessionApi
    {
        fn next_session_authorities() -> Result<Vec<AuthorityId>, ApiError>;
        fn authorities() -> Vec<AuthorityId>;
        fn next_session_authority_data() -> Result<SessionAuthorityData, ApiError>;
        fn authority_data() -> SessionAuthorityData;
        fn session_period() -> u32;
        fn millisecs_per_block() -> u64;
    }
}
