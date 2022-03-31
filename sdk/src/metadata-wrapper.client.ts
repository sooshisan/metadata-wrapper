import * as anchor from "@project-serum/anchor";
import { Program, Provider, Idl, Wallet } from "@project-serum/anchor";
import {
  TOKEN_PROGRAM_ID,
  Token,
  ASSOCIATED_TOKEN_PROGRAM_ID,
  MintLayout,
} from "@solana/spl-token";
import {
  Connection,
  Keypair,
  SystemProgram,
  PublicKey,
  SYSVAR_RENT_PUBKEY,
} from "@solana/web3.js";

import {
  MetadataInfo,
  UpdateMetadataInfo,
  TOKEN_METADATA_PROGRAM_ID,
  AccountUtils,
  getSignersFromPayer,
  decodeMetadata,
  Metadata,
  decodeMasterEdition,
  MasterEditionV2,
  MasterEditionV1,
  getMetadata as _getMetadata,
  getMasterEdition as _getMasterEdition,
} from "./common";
import { MetadataWrapper } from "./types/metadata_wrapper";

export class MetadataWrapperClient extends AccountUtils {
  wallet: Wallet;
  provider!: Provider;
  program!: Program<MetadataWrapper>;

  readonlyKeypair!: Keypair;

  constructor(
    connection: Connection,
    wallet: anchor.Wallet,
    idl?: Idl,
    programId?: PublicKey
  ) {
    super(connection);
    this.wallet = wallet;
    this.setProvider();
    this.setProgram(idl, programId);

    this.readonlyKeypair = Keypair.generate();
  }

  setProvider = () => {
    this.provider = new Provider(
      this.connection,
      this.wallet,
      Provider.defaultOptions()
    );
    anchor.setProvider(this.provider);
  };

  setProgram = (idl?: Idl, programId?: PublicKey) => {
    // instantiating program depends on the environment
    if (idl && programId) {
      console.log("idl: ", idl);
      // means running in prod
      this.program = new Program<MetadataWrapper>(
        idl as any,
        programId,
        this.provider
      );
    } else {
      // means running inside test suite
      this.program = anchor.workspace
        .MetadataWrapper as Program<MetadataWrapper>;
    }
  };

  // ================================================
  // PDAs
  // ================================================
  getMasterEdition = async (mint: PublicKey): Promise<PublicKey> => {
    return _getMasterEdition(mint);
  };

  getMetadata = async (mint: PublicKey): Promise<PublicKey> => {
    return _getMetadata(mint);
  };

  // ================================================
  // Fetch & deserialize objects
  // ================================================

  fetchMetadata = async (metadata: PublicKey): Promise<Metadata> => {
    const accountInfo = await this.connection.getAccountInfo(metadata);

    if (!accountInfo) {
      throw new Error(
        `No account data found for public key: ${metadata.toBase58()}`
      );
    }

    return decodeMetadata(accountInfo.data);
  };

  fetchMasterEdition = async (
    masterEdition: PublicKey
  ): Promise<MasterEditionV1 | MasterEditionV2> => {
    const accountInfo = await this.connection.getAccountInfo(masterEdition);

    if (!accountInfo) {
      throw new Error(
        `No account data found for public key: ${masterEdition.toBase58()}`
      );
    }

    return decodeMasterEdition(accountInfo.data);
  };

  // ================================================
  // Smart contract function helpers
  // ================================================

  generateMintInstructions = async (
    mint: PublicKey,
    owner: PublicKey,
    payer: PublicKey,
    mintAuthority: PublicKey, // must sign as well
    freezeAuthority: PublicKey
  ) => {
    const associatedTokenAddress: PublicKey =
      await this.findAssociatedTokenAddress(owner, mint);
    return [
      anchor.web3.SystemProgram.createAccount({
        fromPubkey: payer,
        newAccountPubkey: mint,
        space: MintLayout.span,
        lamports: await this.connection.getMinimumBalanceForRentExemption(
          MintLayout.span
        ),
        programId: TOKEN_PROGRAM_ID,
      }),
      Token.createInitMintInstruction(
        TOKEN_PROGRAM_ID,
        mint,
        0,
        mintAuthority, // mint authority
        freezeAuthority // freeze authority
      ),
      Token.createAssociatedTokenAccountInstruction(
        ASSOCIATED_TOKEN_PROGRAM_ID,
        TOKEN_PROGRAM_ID,
        mint,
        associatedTokenAddress,
        owner, // owner
        payer // payer
      ),
      Token.createMintToInstruction(
        TOKEN_PROGRAM_ID,
        mint,
        associatedTokenAddress,
        mintAuthority, // signer
        [],
        1
      ),
    ];
  };

  generateMetadataAccounts = async (
    mint: Keypair,
    payer: PublicKey | Keypair
  ) => {
    const signerInfo = getSignersFromPayer(payer);
    const metadata = await this.getMetadata(mint.publicKey);
    const masterEdition = await this.getMasterEdition(mint.publicKey);

    return {
      accounts: {
        payer: signerInfo.payer,
        mint: mint.publicKey,
        mintAuthority: signerInfo.payer,
        updateAuthority: signerInfo.payer,
        metadata,
        masterEdition,
        tokenMetadataProgram: TOKEN_METADATA_PROGRAM_ID,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
        rent: SYSVAR_RENT_PUBKEY,
      },
      signers: signerInfo.signers,
    };
  };

  generateMetadata = async (
    mint: Keypair,
    metadataInfo: MetadataInfo,
    payer: PublicKey | Keypair
  ) => {
    const generateMetadataAccounts = await this.generateMetadataAccounts(
      mint,
      payer
    );

    const _payer = generateMetadataAccounts.accounts.payer;

    const tx = await this.program.rpc.generate(metadataInfo as any, {
      accounts: generateMetadataAccounts.accounts,
      preInstructions: await this.generateMintInstructions(
        mint.publicKey,
        _payer,
        _payer,
        _payer,
        _payer
      ),
      signers: [...generateMetadataAccounts.signers, mint],
    });

    return {
      tx,
      mint: mint.publicKey,
      metadata: generateMetadataAccounts.accounts.metadata,
      masterEdition: metadataInfo.supply
        ? generateMetadataAccounts.accounts.masterEdition
        : undefined,
    };
  };

  signMetadata = async (mint: PublicKey, payer: PublicKey | Keypair) => {
    const signerInfo = getSignersFromPayer(payer);

    const metadata = await this.getMetadata(mint);

    return this.program.rpc.signMetadata({
      accounts: {
        creator: signerInfo.payer,
        metadata,
        tokenMetadataProgram: TOKEN_METADATA_PROGRAM_ID,
      },
      signers: signerInfo.signers,
    });
  };

  updateMetadataUri = async (
    mint: PublicKey,
    uri: string,
    payer: PublicKey | Keypair
  ) => {
    const signerInfo = getSignersFromPayer(payer);
    const metadata = await this.getMetadata(mint);

    return this.program.rpc.updateMetadataUri(uri, {
      accounts: {
        updateAuthority: signerInfo.payer,
        metadata,
        tokenMetadataProgram: TOKEN_METADATA_PROGRAM_ID,
      },
      signers: signerInfo.signers,
    });
  };

  updateMetadata = async (
    mint: PublicKey,
    updateMetadataInfo: UpdateMetadataInfo,
    payer: PublicKey | Keypair
  ) => {
    const signerInfo = getSignersFromPayer(payer);
    const metadata = await this.getMetadata(mint);

    return this.program.rpc.updateMetadata(updateMetadataInfo as any, {
      accounts: {
        updateAuthority: signerInfo.payer,
        metadata,
        tokenMetadataProgram: TOKEN_METADATA_PROGRAM_ID,
      },
      signers: signerInfo.signers,
    });
  };
}
