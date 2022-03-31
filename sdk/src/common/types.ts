import { u64 } from "@solana/spl-token";
import { Keypair, PublicKey } from "@solana/web3.js";

export interface SignerInfo {
  payer: PublicKey;
  signers: Keypair[];
}

// avoid collisions with metaplex Creator struct
export interface MetadataCreator {
    address: PublicKey;
    verified: boolean;
    share: number;
}

export interface MetadataInfo {
  name: string;
  symbol: string;
  uri: string;
  sellerFeeBasisPoints: number; // u16
  updateAuthorityIsSigner: boolean;
  isMutable: boolean;
  creators?: MetadataCreator[]; // Creator
  collection?: PublicKey; // Collection
  uses?: any; // Uses
  supply?: u64; // u64
  newUpdateAuthority?: PublicKey;
}

export interface UpdateMetadataInfo {
  name: string;
  symbol: string;
  uri: string;
  sellerFeeBasisPoints: number; // u16
  creators?: MetadataCreator[]; // Creator
  collection?: PublicKey; // Collection
  uses?: any; // Uses
}
