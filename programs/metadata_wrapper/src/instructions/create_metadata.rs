use {
    crate::MetadataInfo, anchor_lang::prelude::*, anchor_spl::token::Token,
    mpl_token_metadata::instruction::create_metadata_accounts_v2,
    solana_program::program::invoke_signed,
};

#[derive(Accounts)]
pub struct CreateMetadata<'info> {
    /// CHECK: verified via cpi into the metaplex metadata program
    pub payer: AccountInfo<'info>,

    // the following accounts aren't using anchor macros because CPI invocation
    // will do the required validations.
    /// CHECK: verified via cpi into the metaplex metadata program
    pub metadata: AccountInfo<'info>,

    // mint address the token with which metadata will be associated
    /// CHECK: verified via cpi into the metaplex metadata program
    pub mint: AccountInfo<'info>,

    /// CHECK: verified via cpi into the metaplex metadata program
    pub mint_authority: AccountInfo<'info>,

    // account with authority to update metadata, if mutable
    /// CHECK: verified via cpi into the metaplex metadata program
    pub update_authority: AccountInfo<'info>,

    /// CHECK: verified via cpi into the metaplex metadata program
    #[account(address = mpl_token_metadata::id())]
    pub token_metadata_program: UncheckedAccount<'info>,

    #[account(address = spl_token::id())]
    pub token_program: Program<'info, Token>,

    pub system_program: Program<'info, System>,

    pub rent: Sysvar<'info, Rent>,
}

pub fn handle<'a, 'b, 'c, 'info>(
    ctx: CpiContext<'a, 'b, 'c, 'info, CreateMetadata<'info>>,
    metadata_info: MetadataInfo,
) -> ProgramResult {
    msg!("Creating metadata for mint = {}", ctx.accounts.mint.key());

    let creators = metadata_info.clone().to_mpl_creators();
    let collection = metadata_info.to_mpl_collection();
    let uses = metadata_info.to_mpl_uses();

    invoke_signed(
        &create_metadata_accounts_v2(
            *ctx.accounts.token_metadata_program.key,
            *ctx.accounts.metadata.key,
            ctx.accounts.mint.key(),
            ctx.accounts.mint_authority.key(),
            ctx.accounts.payer.key(),
            ctx.accounts.update_authority.key(),
            metadata_info.name,
            metadata_info.symbol,
            metadata_info.uri,
            creators,
            metadata_info.seller_fee_basis_points,
            metadata_info.update_authority_is_signer,
            metadata_info.is_mutable,
            collection,
            uses,
        ),
        &[
            ctx.accounts.metadata.clone(),
            ctx.accounts.mint.to_account_info(),
            ctx.accounts.mint_authority.to_account_info(),
            ctx.accounts.payer.to_account_info(),
            ctx.accounts.update_authority.to_account_info(),
            ctx.accounts.system_program.to_account_info(),
            ctx.accounts.rent.to_account_info(),
        ],
        ctx.signer_seeds,
    )?;

    Ok(())
}
