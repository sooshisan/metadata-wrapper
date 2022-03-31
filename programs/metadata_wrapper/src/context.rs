use {
    crate::{
        instructions::{
            create_master_edition::CreateMasterEdition, create_metadata::CreateMetadata,
            sign_metadata::SignMetadata, update_metadata::UpdateMetadata,
        },
        MetadataWrapper,
    },
    anchor_lang::{prelude::*, solana_program::system_program},
    anchor_spl::token::{Mint, Token},
};

#[derive(Accounts)]
pub struct GenerateContext<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    pub mint_authority: Signer<'info>,

    pub update_authority: Signer<'info>,

    #[account(
        mut,
        constraint = mint.decimals == 0,
        constraint = mint.supply == 1,
        constraint = mint.freeze_authority.unwrap() == update_authority.key(),
        constraint = mint.mint_authority.unwrap() == update_authority.key(),
    )]
    pub mint: Account<'info, Mint>,

    /// CHECK: verified via cpi into the metaplex metadata program
    #[account(mut)]
    pub metadata: AccountInfo<'info>,

    /// CHECK: verified via cpi into the metaplex metadata program
    #[account(mut)]
    pub master_edition: AccountInfo<'info>,

    /// CHECK: verified via cpi into the metaplex metadata program
    #[account(address = mpl_token_metadata::id())]
    pub token_metadata_program: UncheckedAccount<'info>,

    #[account(address = spl_token::id())]
    pub token_program: Program<'info, Token>,

    #[account(address = system_program::id())]
    pub system_program: Program<'info, System>,

    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct SignMetadataContext<'info> {
    pub creator: Signer<'info>,

    /// no need to check seeds here because account will be verified via CPI call into metaplex metadata contract
    #[account(mut)]
    pub metadata: Account<'info, MetadataWrapper>,

    /// CHECK: verified via cpi into the metaplex metadata program
    #[account(address = spl_token_metadata::id())]
    pub token_metadata_program: AccountInfo<'info>,
}

#[derive(Accounts)]
pub struct UpdateMetadataContext<'info> {
    #[account(mut)]
    pub update_authority: Signer<'info>,

    /// no need to check seeds here because account will be verified via CPI call into metaplex metadata contract
    #[account(mut)]
    pub metadata: Account<'info, MetadataWrapper>,

    /// CHECK: verified via cpi into the metaplex metadata program
    #[account(address = spl_token_metadata::id())]
    pub token_metadata_program: AccountInfo<'info>,
}

// ==============================================
// cpi context transformations
// ==============================================

impl<'info> GenerateContext<'info> {
    pub fn into_create_metadata_context(
        &self,
    ) -> CpiContext<'_, '_, '_, 'info, CreateMetadata<'info>> {
        let cpi_program = self.token_metadata_program.to_account_info();

        let cpi_accounts = CreateMetadata {
            metadata: self.metadata.to_account_info(),
            mint: self.mint.to_account_info(),
            mint_authority: self.mint_authority.to_account_info(),
            payer: self.payer.to_account_info(),
            update_authority: self.update_authority.to_account_info(),
            token_metadata_program: self.token_metadata_program.clone(),
            token_program: self.token_program.clone(),
            system_program: self.system_program.clone(),
            rent: self.rent.clone(),
        };

        CpiContext::new(cpi_program, cpi_accounts)
    }

    pub fn into_create_master_edition_metadata_context(
        &self,
    ) -> CpiContext<'_, '_, '_, 'info, CreateMasterEdition<'info>> {
        let cpi_program = self.token_metadata_program.to_account_info();

        // is mint_authority & update_authority = payer / minter?
        // yes — https://github.com/exiled-apes/candy-machine-mint/blob/main/src/candy-machine.ts#L268-L269
        let cpi_accounts = CreateMasterEdition {
            payer: self.payer.to_account_info(),
            metadata: self.metadata.to_account_info(),
            master_edition: self.master_edition.to_account_info(),
            mint: self.mint.to_account_info(),
            mint_authority: self.mint_authority.to_account_info(),
            update_authority: self.update_authority.to_account_info(),
            token_metadata_program: self.token_metadata_program.clone(),
            token_program: self.token_program.clone(),
            system_program: self.system_program.clone(),
            rent: self.rent.clone(),
        };

        CpiContext::new(cpi_program, cpi_accounts)
    }

    pub fn into_update_metadata_context(
        &self,
    ) -> CpiContext<'_, '_, '_, 'info, UpdateMetadata<'info>> {
        let cpi_program = self.token_metadata_program.to_account_info();

        let cpi_accounts = UpdateMetadata {
            metadata: self.metadata.to_account_info(),
            update_authority: self.update_authority.to_account_info(),
            token_metadata_program: self.token_metadata_program.to_account_info(),
        };

        CpiContext::new(cpi_program, cpi_accounts)
    }
}

impl<'info> SignMetadataContext<'info> {
    pub fn into_sign_metadata_context(&self) -> CpiContext<'_, '_, '_, 'info, SignMetadata<'info>> {
        let cpi_program = self.token_metadata_program.to_account_info();

        let cpi_accounts = SignMetadata {
            metadata: self.metadata.to_account_info(),
            creator: self.creator.to_account_info(),
            token_metadata_program: self.token_metadata_program.to_account_info(),
        };

        CpiContext::new(cpi_program, cpi_accounts)
    }
}

impl<'info> UpdateMetadataContext<'info> {
    pub fn into_update_metadata_context(
        &self,
    ) -> CpiContext<'_, '_, '_, 'info, UpdateMetadata<'info>> {
        let cpi_program = self.token_metadata_program.to_account_info();

        let cpi_accounts = UpdateMetadata {
            metadata: self.metadata.to_account_info(),
            update_authority: self.update_authority.to_account_info(),
            token_metadata_program: self.token_metadata_program.to_account_info(),
        };

        CpiContext::new(cpi_program, cpi_accounts)
    }
}
