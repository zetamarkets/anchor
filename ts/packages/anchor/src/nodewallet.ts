import { Buffer } from "buffer";
import {
  Keypair,
  PublicKey,
  Transaction,
  VersionedTransaction,
} from "@solana/web3.js";
import { Wallet, isVersionedTransaction } from "./provider";

/**
 * Node only wallet.
 */
export default class NodeWallet implements Wallet {
  constructor(readonly payer: Keypair) {}

  static local(): NodeWallet | never {
    const process = require("process");

    if (!process.env.ANCHOR_WALLET || process.env.ANCHOR_WALLET === "") {
      throw new Error(
        "expected environment variable `ANCHOR_WALLET` is not set."
      );
    }

    const payer = Keypair.fromSecretKey(
      Buffer.from(
        JSON.parse(
          require("fs").readFileSync(process.env.ANCHOR_WALLET, {
            encoding: "utf-8",
          })
        )
      )
    );

    return new NodeWallet(payer);
  }

  async signTransaction(
    tx: Transaction | VersionedTransaction
  ): Promise<Transaction | VersionedTransaction> {
    if (isVersionedTransaction(tx)) {
      tx.sign([this.payer]);
    } else {
      tx.partialSign(this.payer);
    }

    return tx;
  }

  async signAllTransactions(
    txs: Transaction[] | VersionedTransaction[]
  ): Promise<Transaction[] | VersionedTransaction[]> {
    return txs.map((t) => {
      if (isVersionedTransaction(t)) {
        t.sign([this.payer]);
      } else {
        t.partialSign(this.payer);
      }
      return t;
    });
  }

  get publicKey(): PublicKey {
    return this.payer.publicKey;
  }
}
