import * as anchor from "@project-serum/anchor";

import { expect } from "chai";
import {
  PublicKey,
  Keypair,
  Connection,
  LAMPORTS_PER_SOL,
} from "@solana/web3.js";
import { u64 } from "@solana/spl-token";

import {
  MetadataWrapperClient,
  MetadataInfo,
  Creator,
  NodeWallet,
  UpdateMetadataInfo,
  MasterEditionV1,
  MasterEditionV2,
  Metadata,
  MetadataCreator
} from "../sdk";

import { expectThrowsAsync } from "./util";

export const generateDefaultCreators = (
  authority: PublicKey,
  other: Keypair = null
) => [
  {
    address: authority,
    verified: true,
    share: 60,
  } as MetadataCreator,
  {
    address:
      other === null
        ? Keypair.generate().publicKey
        : other.publicKey,
    verified: false,
    share: 40,
  } as MetadataCreator,
];

export const generateMetadata = (
  name: string,
  symbol: string,
  uri: string,
  supply: number | null,
  authority: PublicKey,
  creators = generateDefaultCreators(authority),
  newUpdateAuthority: PublicKey = null
): MetadataInfo => {
  return {
    name,
    symbol,
    uri,
    sellerFeeBasisPoints: 420,
    updateAuthorityIsSigner: true,
    isMutable: true,
    creators,
    supply: supply !== null ? new u64(supply) : null,
    collection: null,
    uses: null,
    newUpdateAuthority,
  };
};

export const accountExists = async (
  connection: Connection,
  address: PublicKey
): Promise<boolean> => {
  try {
    await connection.getAccountInfo(address);
    return true;
  } catch (err: any) {
    return false;
  }
};

export const validateMasterEdition = async (
  actualMasterEdition: MasterEditionV1 | MasterEditionV2,
  expectedMaxSupply: number
) => {
  expect(actualMasterEdition.maxSupply.toNumber()).to.be.equal(
    expectedMaxSupply
  );
};

export interface ExpectedMetadataAttributes {
  name: string;
  symbol: string;
  primarySaleHappened: boolean;
  updateAuthority: PublicKey;
  isMutable: boolean;
  creators?: Creator[];
}

export const validateMetadata = async (
  actualMetadata: Metadata,
  expectedMetadata: ExpectedMetadataAttributes
) => {
  expect(expectedMetadata.symbol).to.be.equal(actualMetadata.data.symbol);
  expect(expectedMetadata.name).to.be.equal(actualMetadata.data.name);
  expect(expectedMetadata.symbol).to.be.equal(actualMetadata.data.symbol);
  expect(!!actualMetadata.primarySaleHappened).to.be.equal(
    expectedMetadata.primarySaleHappened
  );
  expect(actualMetadata.updateAuthority).to.be.equal(
    expectedMetadata.updateAuthority.toBase58()
  );
  expect(!!actualMetadata.isMutable).to.be.equal(expectedMetadata.isMutable);

  if (expectedMetadata.creators) {
    expectedMetadata.creators.forEach((c) => {
      const _actual = actualMetadata.data.creators.filter(
        (_c) => _c.address === c.address
      )[0];

      // no need to check address by virtue of filter
      expect(_actual.share).to.be.equal(c.share);
      expect(!!_actual.verified).to.be.equal(c.verified);
    });
  }
};

describe("metadata_wrapper", () => {
  const _provider = anchor.Provider.env();

  const client = new MetadataWrapperClient(
    _provider.connection,
    _provider.wallet as anchor.Wallet
  );

  const nodeWallet = new NodeWallet(
    anchor.Provider.env().connection,
    anchor.Provider.env().wallet as anchor.Wallet
  );

  let user: Keypair;
  let otherCreator: Keypair;
  let rando: Keypair;

  let defaultCreators: MetadataCreator[];
  const mint1: Keypair = Keypair.generate();
  const mint2: Keypair = Keypair.generate();

  before("fund user wallets", async () => {
    user = await nodeWallet.createFundedWallet(2 * LAMPORTS_PER_SOL);
    otherCreator = await nodeWallet.createFundedWallet(2 * LAMPORTS_PER_SOL);
    rando = await nodeWallet.createFundedWallet(2 * LAMPORTS_PER_SOL);
  });

  before("setup default creators", () => {
    defaultCreators = generateDefaultCreators(user.publicKey, otherCreator);
  });

  it("Generate metadata happy path => metadata, master edition, primary sale happened", async () => {
    const name = "NAMO";
    const symbol = "SYMB";
    const uri = "https://someserver.com";
    const maxSupply = 0;

    const metadataInfo = generateMetadata(
      name,
      symbol,
      uri,
      maxSupply,
      user.publicKey,
      defaultCreators
    );

    const {
      mint: mintPubkey,
      metadata,
      masterEdition,
    } =  await client.generateMetadata(mint1, metadataInfo, user);

    expect(await accountExists(client.connection, mintPubkey)).to.be.true;
    expect(mint1.publicKey.toBase58()).to.be.equal(mintPubkey.toBase58());

    expect(await accountExists(client.connection, metadata)).to.be.true;
    const _metadata = await client.fetchMetadata(metadata);
    validateMetadata(_metadata, {
      name,
      symbol,
      primarySaleHappened: true,
      updateAuthority: user.publicKey,
      isMutable: true,
    });

    expect(await accountExists(client.connection, masterEdition)).to.be.true;
    const _masterEdition = await client.fetchMasterEdition(masterEdition);
    validateMasterEdition(_masterEdition, maxSupply);
  });

  it("Other creator can sign metadata", async () => {
    const metadata = await client.getMetadata(mint1.publicKey);
    const _metadataBefore = await client.fetchMetadata(metadata);

    await client.signMetadata(mint1.publicKey, otherCreator);

    const _metadataAfter = await client.fetchMetadata(metadata);
    const _before = !!_metadataBefore.data.creators.filter(
      (c) => c.address === otherCreator.publicKey.toBase58()
    )[0].verified;
    const _after = !!_metadataAfter.data.creators.filter(
      (c) => c.address === otherCreator.publicKey.toBase58()
    )[0].verified;

    expect(!_before).to.be.equal(_after);
    expect(_after).to.be.equal(true);
  });

  it("Non update authority cannot modify URI", async () => {
    expectThrowsAsync(() =>
      client.updateMetadataUri(mint1.publicKey, "somenewuri", rando)
    );
  });

  it("Update authority can modify URI", async () => {
    const metadata = await client.getMetadata(mint1.publicKey);
    const _metadataBefore = await client.fetchMetadata(metadata);

    const uri2 = "https://helloworldserver.com";
    await client.updateMetadataUri(mint1.publicKey, uri2, user);

    const _metadataAfter = await client.fetchMetadata(metadata);

    expect(_metadataBefore.data.uri).to.not.equal(_metadataAfter.data.uri);
    expect(_metadataAfter.data.uri).to.be.equal(uri2);
  });

  it("Non update authority cannot modify metadata", async () => {
    const updateMetadataInfo = {
      name: "name2",
      symbol: "symbol2",
      uri: "uri2",
      sellerFeeBasisPoints: 1,
      creators: null,
      collection: null,
      uses: null,
    } as UpdateMetadataInfo;

    expectThrowsAsync(() =>
      client.updateMetadata(mint1.publicKey, updateMetadataInfo, rando)
    );
  });

  it("Update authority can modify all metadata", async () => {
    const metadata = await client.getMetadata(mint1.publicKey);
    const _metadataBefore = await client.fetchMetadata(metadata);

    const updateMetadataInfo = {
      name: "name2",
      symbol: "symbol2",
      uri: "uri2",
      sellerFeeBasisPoints: 1,
      creators: null,
      collection: null,
      uses: null,
    } as UpdateMetadataInfo;

    await client.updateMetadata(mint1.publicKey, updateMetadataInfo, user);

    const _metadataAfter = await client.fetchMetadata(metadata);
    expect(_metadataBefore.data.name).to.not.equal(_metadataAfter.data.name);
    expect(_metadataAfter.data.name).to.be.equal(updateMetadataInfo.name);
    expect(_metadataBefore.data.symbol).to.not.equal(
      _metadataAfter.data.symbol
    );
    expect(_metadataAfter.data.symbol).to.be.equal(updateMetadataInfo.symbol);
    expect(_metadataBefore.data.uri).to.not.equal(_metadataAfter.data.uri);
    expect(_metadataAfter.data.uri).to.be.equal(updateMetadataInfo.uri);
    expect(_metadataBefore.data.sellerFeeBasisPoints).to.not.equal(
      _metadataAfter.data.sellerFeeBasisPoints
    );
    expect(_metadataAfter.data.sellerFeeBasisPoints).to.be.equal(
      updateMetadataInfo.sellerFeeBasisPoints
    );
    expect(_metadataAfter.data.creators).to.be.equal(undefined);
  });

  it("Generate metadata with new update authority, verify original payer cannot then modify metadata", async () => {
    const metadataInfo = generateMetadata(
      "name1",
      "symbol1",
      "uri1",
      0,
      user.publicKey,
      defaultCreators,
      otherCreator.publicKey
    );

    const { metadata } = await client.generateMetadata(
      mint2,
      metadataInfo,
      user
    );
    const _metadata = await client.fetchMetadata(metadata);
    // expect update authority to be updated from user -> otherCreator
    expect(_metadata.updateAuthority).to.be.equal(
      otherCreator.publicKey.toBase58()
    );

    // original payer attempts to update uri, fails. but, new update authority updates uri.
    expectThrowsAsync(() =>
      client.updateMetadataUri(mint2.publicKey, "userUri", user)
    );

    await client.updateMetadataUri(
      mint2.publicKey,
      "otherCreatorUri",
      otherCreator
    );
  });

  it("Generate metadata without master edition", async () => {
    const name = "name1";
    const symbol = "symbol1";
    const uri = "uri1";

    const metadataInfo = generateMetadata(
      name,
      symbol,
      uri,
      null,
      user.publicKey,
      defaultCreators
    );

    const mint = Keypair.generate();

    const {
      mint: mintPubkey,
      metadata,
      masterEdition,
    } = await client.generateMetadata(mint, metadataInfo, user);

    expect(await accountExists(client.connection, mintPubkey)).to.be.true;
    expect(await accountExists(client.connection, metadata)).to.be.true;
    const _metadata = await client.fetchMetadata(metadata);
    validateMetadata(_metadata, {
      name,
      symbol,
      primarySaleHappened: true,
      updateAuthority: user.publicKey,
      isMutable: true,
    });

    // no master edition exists
    expect(masterEdition === undefined).to.be.true;
  });
});
