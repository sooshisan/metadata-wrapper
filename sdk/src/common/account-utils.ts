// imported from gemworks
// source: https://github.com/gemworks/gem-farm/blob/main/src/gem-common/account-utils.ts
import { Connection, PublicKey } from "@solana/web3.js";
import {
  ASSOCIATED_TOKEN_PROGRAM_ID,
  TOKEN_PROGRAM_ID,
} from "@solana/spl-token";

export class AccountUtils {
  connection: Connection;

  constructor(connection: Connection) {
    this.connection = connection;
  }

  findProgramAddress = async (
    programId: PublicKey,
    seeds: (PublicKey | Uint8Array | string)[]
  ): Promise<[PublicKey, number]> => {
    const seed_bytes = seeds.map((s) => {
      if (typeof s === "string") {
        return Buffer.from(s);
      } else if ("toBytes" in s) {
        return s.toBytes();
      } else {
        return s;
      }
    });

    return await PublicKey.findProgramAddress(seed_bytes, programId);
  };

  findAssociatedTokenAddress = async (
    owner: PublicKey,
    mint: PublicKey
  ): Promise<PublicKey> => {
    return (
      await PublicKey.findProgramAddress(
        [owner.toBuffer(), TOKEN_PROGRAM_ID.toBuffer(), mint.toBuffer()],
        ASSOCIATED_TOKEN_PROGRAM_ID
      )
    )[0];
  };
}
