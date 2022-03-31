use anchor_lang::prelude::*;

pub mod context;
pub mod instructions;

pub use context::*;
pub use instructions::*;
pub use instructions::update_metadata::UpdateMetadataInfo;

use {
    anchor_lang::solana_program::borsh::try_from_slice_unchecked,
    borsh::{BorshDeserialize, BorshSerialize},
    mpl_token_metadata::{
        state::{Key as MetaplexKey, Metadata, MAX_METADATA_LEN},
        utils::try_from_slice_checked,
    },
    std::ops::{Deref, DerefMut}
};

declare_id!("FGDFtTf13pWintava7VXbpytXWDmXoxVKP2w4fafHhQQ");

#[program]
pub mod metadata_wrapper {
    use super::*;

    // main instruction; allows caller to
    // 1. create metadata for a given mint,
    // 2. optionally create a master edition, aand
    // 3. toggle primary sale happened, optionally modify update authority
    pub fn generate(ctx: Context<GenerateContext>, metadata_info: MetadataInfo) -> ProgramResult {
        let token_supply: Option<u64> = metadata_info.supply;
        let new_update_authority: Option<Pubkey> = metadata_info.new_update_authority;

        instructions::create_metadata::handle(
            ctx.accounts.into_create_metadata_context(),
            metadata_info,
        )?;

        match token_supply {
            Some(supply) => instructions::create_master_edition::handle(
                ctx.accounts.into_create_master_edition_metadata_context(),
                supply,
            )?,
            None => (),
        };

        instructions::update_metadata::update_for_primary_sale(
            ctx.accounts.into_update_metadata_context(),
            new_update_authority,
        )?;

        Ok(())
    }

    // simple passthrough instruction to allow any creator to verify themselves
    pub fn sign_metadata(ctx: Context<SignMetadataContext>) -> ProgramResult {
        instructions::sign_metadata::handle(ctx.accounts.into_sign_metadata_context())?;

        Ok(())
    }

    // a
    pub fn update_metadata_uri(ctx: Context<UpdateMetadataContext>, uri: String) -> ProgramResult {
        instructions::update_metadata::update_uri(
            ctx.accounts.into_update_metadata_context(),
            uri,
        )?;

        Ok(())
    }

    // update authority can change associated metadata; use with extreme caution because it's possible to overwrite important values, like creator array.
    // will fail if is_mutable = false. source: https://github.com/metaplex-foundation/metaplex-program-library/blob/ddb247622dcfd7501f6811007fbbb88b1bce1483/token-metadata/program/src/processor.rs#L249-L309
    pub fn update_metadata(
        ctx: Context<UpdateMetadataContext>,
        data: UpdateMetadataInfo,
    ) -> ProgramResult {
        instructions::update_metadata::update_data(
            ctx.accounts.into_update_metadata_context(),
            data,
        )?;

        Ok(())
    }

    // more instructions can be added, but at some point it would be almost the same as interfacing
    // directly with the metaplex metadata contract.
}

// =================================================================================================
// temporarily ported from metaplex metadata lib. copied over to output to anchor idl.
// otherwise, idl can not be parsed due to missing types imported from metaplex. likely a result
// of my dir structure. might refactor to break into all separate anchor directories.
// =================================================================================================

#[repr(C)]
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Default, PartialEq, Debug)]
pub struct ModifyMetadataUriInfo {
    pub uri: String,
}

#[repr(C)]
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Default, PartialEq, Debug)]
pub struct MetadataInfo {
    pub name: String,
    pub symbol: String,
    pub uri: String,
    pub creators: Option<Vec<Creator>>,
    pub seller_fee_basis_points: u16,
    pub update_authority_is_signer: bool,
    pub is_mutable: bool,
    pub collection: Option<Collection>,
    pub uses: Option<Uses>,
    pub supply: Option<u64>,
    pub new_update_authority: Option<Pubkey>,
}

impl MetadataInfo {
    fn to_mpl_creators(&self) -> Option<Vec<mpl_token_metadata::state::Creator>> {
        return match &self.creators {
            Some(creators) => {
                let mut mpl_creators = vec![];

                for creator in creators.iter() {
                    mpl_creators.push(mpl_token_metadata::state::Creator {
                        address: creator.address,
                        share: creator.share,
                        verified: creator.verified,
                    });
                }

                return Some(mpl_creators);
            }
            None => None,
        };
    }

    fn to_mpl_collection(&self) -> Option<mpl_token_metadata::state::Collection> {
        return match &self.collection {
            Some(collection) => Some(mpl_token_metadata::state::Collection {
                key: collection.key,
                verified: collection.verified,
            }),
            None => None,
        };
    }

    fn to_mpl_uses(&self) -> Option<mpl_token_metadata::state::Uses> {
        return match &self.uses {
            Some(uses) => Some(mpl_token_metadata::state::Uses {
                use_method: to_mpl_use_method(&uses.use_method),
                remaining: uses.remaining,
                total: uses.total,
            }),
            None => None,
        };
    }
}

fn to_mpl_use_method(use_method: &UseMethod) -> mpl_token_metadata::state::UseMethod {
    return match use_method {
        UseMethod::Burn => mpl_token_metadata::state::UseMethod::Burn,
        UseMethod::Multiple => mpl_token_metadata::state::UseMethod::Multiple,
        UseMethod::Single => mpl_token_metadata::state::UseMethod::Single,
    };
}

#[repr(C)]
#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Debug)]
pub enum UseMethod {
    Burn,
    Multiple,
    Single,
}

impl Default for UseMethod {
    fn default() -> Self {
        UseMethod::Single
    }
}

#[repr(C)]
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Default, PartialEq, Debug)]
pub struct Uses {
    // 17 bytes + Option byte
    pub use_method: UseMethod, //1
    pub remaining: u64,        //8
    pub total: u64,            //8
}

#[repr(C)]
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Default, PartialEq, Debug)]
pub struct Collection {
    pub verified: bool,
    pub key: Pubkey,
}

#[repr(C)]
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Default, PartialEq, Debug)]
pub struct Creator {
    pub address: Pubkey,
    pub verified: bool,
    // In percentages, NOT basis points ;) Watch out!
    pub share: u8,
}

#[repr(C)]
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Default, PartialEq, Debug)]
pub struct DataV2 {
    /// The name of the asset
    pub name: String,
    /// The symbol for the asset
    pub symbol: String,
    /// URI pointing to JSON representing the asset
    pub uri: String,
    /// Royalty basis points that goes to creators in secondary sales (0-10000)
    pub seller_fee_basis_points: u16,
    /// Array of creators, optional
    pub creators: Option<Vec<Creator>>,
    /// Collection
    pub collection: Option<Collection>,
    /// Uses
    pub uses: Option<Uses>,
}

// =================================================================================================
// custom struct to wrap mpl_token_metadata Metadata so that we can use in anchor context. can also
// be used by any downstream clients using this code.
// https://github.com/emilson0407/sol-nft-staking/blob/799b497ad5bd55978317b105cfbd8a7cffe4e4a1/programs/sol-nft-staking/src/anchor_metaplex.rs
// =================================================================================================

#[derive(Clone)]
pub struct MetadataWrapper(mpl_token_metadata::state::Metadata);

impl MetadataWrapper {
    pub const LEN: usize = MAX_METADATA_LEN;
}

impl anchor_lang::AccountDeserialize for MetadataWrapper {
    fn try_deserialize(buf: &mut &[u8]) -> Result<Self, ProgramError> {
        try_from_slice_checked(buf, MetaplexKey::MetadataV1, MAX_METADATA_LEN).map(MetadataWrapper)
    }

    fn try_deserialize_unchecked(buf: &mut &[u8]) -> Result<Self, ProgramError> {
        let data: &[u8] = &buf[8..];
        let metadata: Metadata = try_from_slice_unchecked(data)
            .map_err(|err| ProgramError::BorshIoError(err.to_string()))?;
        Ok(MetadataWrapper(metadata))
    }
}

impl anchor_lang::AccountSerialize for MetadataWrapper {
    fn try_serialize<W: std::io::Write>(&self, _writer: &mut W) -> Result<(), ProgramError> {
        // no-op
        Ok(())
    }
}

impl anchor_lang::Owner for MetadataWrapper {
    fn owner() -> Pubkey {
        mpl_token_metadata::ID
    }
}

impl Deref for MetadataWrapper {
    type Target = Metadata;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for MetadataWrapper {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
