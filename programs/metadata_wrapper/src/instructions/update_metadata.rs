use {
    anchor_lang::{prelude::*, solana_program::system_program},
    mpl_token_metadata::{
        instruction::update_metadata_accounts_v2,
        state::{Collection, Creator, DataV2, Metadata, Uses},
    },
    solana_program::{
        instruction::Instruction,
        program::{invoke, invoke_signed},
    },
};

#[repr(C)]
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Default, PartialEq, Debug)]
pub struct UpdateMetadataInfo {
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

#[derive(Accounts)]
pub struct UpdateMetadata<'info> {
    /// CHECK: verified via cpi into the metaplex metadata program
    #[account(mut)]
    pub metadata: AccountInfo<'info>,

    /// CHECK: verified via cpi into the metaplex metadata program
    pub update_authority: AccountInfo<'info>,

    /// CHECK: verified via cpi into the metaplex metadata program
    #[account(address = spl_token_metadata::id())]
    pub token_metadata_program: AccountInfo<'info>,
}

pub fn update_for_primary_sale<'a, 'b, 'c, 'info>(
    ctx: CpiContext<'a, 'b, 'c, 'info, UpdateMetadata<'info>>,
    new_update_authority: Option<Pubkey>,
) -> ProgramResult {
    let update_authority: Pubkey = match new_update_authority {
        Some(authority) => authority,
        None => ctx.accounts.update_authority.key(),
    };

    msg!(
        "Updating metadata to indicate primary sale happened, with update authority = {}",
        update_authority
    );

    handle(
        ctx,
        Some(update_authority),
        None,
        Some(true),
    )?;

    Ok(())
}

// only update uri
pub fn update_uri<'a, 'b, 'c, 'info>(
    ctx: CpiContext<'a, 'b, 'c, 'info, UpdateMetadata<'info>>,
    uri: String,
) -> ProgramResult {
    msg!("yay, we reached the update uri function!");
    // https://github.com/metaplex-foundation/metaplex-program-library/blob/ddb247622dcfd7501f6811007fbbb88b1bce1483/token-metadata/program/src/processor.rs#L261
    let metadata = Metadata::from_account_info(&ctx.accounts.metadata)?;

    let data_v2 = DataV2 {
        name: metadata.data.name,
        symbol: metadata.data.symbol,
        uri: uri.to_string(), // updated uri
        seller_fee_basis_points: metadata.data.seller_fee_basis_points,
        creators: metadata.data.creators,
        collection: metadata.collection,
        uses: metadata.uses,
    };

    handle(ctx, None, Some(data_v2), None)?;

    Ok(())
}

// completely update metadata
pub fn update_data<'a, 'b, 'c, 'info>(
    ctx: CpiContext<'a, 'b, 'c, 'info, UpdateMetadata<'info>>,
    data: UpdateMetadataInfo,
) -> ProgramResult {
    let data_v2 = DataV2 {
        name: data.name,
        symbol: data.symbol,
        uri: data.uri,
        seller_fee_basis_points: data.seller_fee_basis_points,
        creators: data.creators,
        collection: data.collection,
        uses: data.uses,
    };

    handle(ctx, None, Some(data_v2), None)?;

    Ok(())
}

// shared handler function, not to be called publically
fn handle<'a, 'b, 'c, 'info>(
    ctx: CpiContext<'a, 'b, 'c, 'info, UpdateMetadata<'info>>,
    update_authority: Option<Pubkey>,
    data: Option<DataV2>,
    primary_sale_happened: Option<bool>,
) -> ProgramResult {
    let ix: &Instruction = &update_metadata_accounts_v2(
        *ctx.accounts.token_metadata_program.key,
        *ctx.accounts.metadata.key,
        *ctx.accounts.update_authority.key,
        update_authority,
        data,
        primary_sale_happened,
        None, // no change to is_mutable
    );

    let update_authority_account_info = ctx.accounts.update_authority.to_account_info();

    // if account is owned by system program && is not executable, we assume that it's a user account, which signed the transaction.
    // in this case, we can just use invoke. otherwise, we need to call invoke_signed with callers signer_seeds.
    // (question) is this check sufficient or should i also check ctx.signer_seeds length? presumably if that's
    // non-zero, some other entity is trying to sign.
    if update_authority_account_info.owner.key() == system_program::ID
        && update_authority_account_info.executable == false
    {
        invoke(
            ix,
            &[
                ctx.accounts.token_metadata_program.to_account_info(),
                ctx.accounts.metadata.to_account_info(),
                update_authority_account_info,
            ],
        )?;
    } else {
        invoke_signed(
            ix,
            &[
                ctx.accounts.token_metadata_program.to_account_info(),
                ctx.accounts.metadata.to_account_info(),
                update_authority_account_info,
            ],
            ctx.signer_seeds,
        )?;
    }

    Ok(())
}
