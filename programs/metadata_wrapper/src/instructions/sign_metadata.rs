use {
    anchor_lang::prelude::*, mpl_token_metadata::instruction::sign_metadata,
    solana_program::program::invoke,
};

#[derive(Accounts)]
pub struct SignMetadata<'info> {
    /// CHECK: verified via cpi into the metaplex metadata program
    #[account(mut)]
    pub metadata: AccountInfo<'info>,

    /// CHECK: verified via cpi into the metaplex metadata program
    pub creator: AccountInfo<'info>,

    /// CHECK: verified via cpi into the metaplex metadata program
    #[account(address = spl_token_metadata::id())]
    pub token_metadata_program: AccountInfo<'info>,
}

pub fn handle<'a, 'b, 'c, 'info>(
    ctx: CpiContext<'a, 'b, 'c, 'info, SignMetadata<'info>>,
) -> ProgramResult {
    invoke(
        &sign_metadata(
            *ctx.accounts.token_metadata_program.key,
            *ctx.accounts.metadata.key,
            *ctx.accounts.creator.key,
        ),
        &[
            ctx.accounts.token_metadata_program.to_account_info(),
            ctx.accounts.metadata.to_account_info(),
            ctx.accounts.creator.to_account_info(),
        ],
    )?;

    Ok(())
}
