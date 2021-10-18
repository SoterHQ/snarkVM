// Copyright (C) 2019-2021 Aleo Systems Inc.
// This file is part of the snarkVM library.

// The snarkVM library is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// The snarkVM library is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with the snarkVM library. If not, see <https://www.gnu.org/licenses/>.

use crate::prelude::*;
use snarkvm_algorithms::{
    merkle_tree::{MerklePath, MerkleTree},
    prelude::*,
};
use snarkvm_utilities::{FromBytes, ToBytes};

use anyhow::{anyhow, Result};
use std::{
    io::{Read, Result as IoResult, Write},
    sync::Arc,
};

/// A ledger proof of inclusion.
#[derive(Derivative)]
#[derivative(Clone(bound = "N: Network"), Debug(bound = "N: Network"))]
pub struct LedgerProof<N: Network> {
    block_hash: N::BlockHash,
    previous_block_hash: N::BlockHash,
    block_header_root: N::BlockHeaderRoot,
    block_header_inclusion_proof: MerklePath<N::BlockHeaderRootParameters>,
    transactions_root: N::TransactionsRoot,
    transactions_inclusion_proof: MerklePath<N::TransactionsRootParameters>,
    transaction_id: N::TransactionID,
    transaction_inclusion_proof: MerklePath<N::TransactionIDParameters>,
    transition_id: N::TransitionID,
    transition_inclusion_proof: MerklePath<N::TransitionIDParameters>,
    commitment: N::Commitment,
}

impl<N: Network> LedgerProof<N> {
    ///
    /// Initializes a new instance of `LedgerProof`.
    ///
    pub fn new(
        block_hash: N::BlockHash,
        previous_block_hash: N::BlockHash,
        block_header_root: N::BlockHeaderRoot,
        block_header_inclusion_proof: MerklePath<N::BlockHeaderRootParameters>,
        transactions_root: N::TransactionsRoot,
        transactions_inclusion_proof: MerklePath<N::TransactionsRootParameters>,
        transaction_id: N::TransactionID,
        transaction_inclusion_proof: MerklePath<N::TransactionIDParameters>,
        transition_id: N::TransitionID,
        transition_inclusion_proof: MerklePath<N::TransitionIDParameters>,
        commitment: N::Commitment,
    ) -> Result<Self> {
        // Ensure the transition inclusion proof is valid.
        if !transition_inclusion_proof.verify(&transition_id, &commitment)? {
            return Err(anyhow!(
                "Commitment {} does not belong to transition {}",
                commitment,
                transition_id
            ));
        }

        // Ensure the transaction inclusion proof is valid.
        if !transaction_inclusion_proof.verify(&transaction_id, &transition_id)? {
            return Err(anyhow!(
                "Transition {} does not belong to transaction {}",
                transition_id,
                transaction_id
            ));
        }

        // Ensure the transactions inclusion proof is valid.
        if !transactions_inclusion_proof.verify(&transactions_root, &transaction_id)? {
            return Err(anyhow!(
                "Transaction {} does not belong to transactions root {}",
                transaction_id,
                transactions_root
            ));
        }

        // Ensure the block header inclusion proof is valid.
        if !block_header_inclusion_proof.verify(&block_header_root, &transactions_root)? {
            return Err(anyhow!(
                "Transactions root {} does not belong to block header {}",
                transactions_root,
                block_header_root
            ));
        }

        // Ensure the block hash is valid.
        let candidate_block_hash = N::block_hash_crh()
            .hash(&[previous_block_hash.to_bytes_le()?, block_header_root.to_bytes_le()?].concat())?;
        if candidate_block_hash != block_hash {
            return Err(anyhow!(
                "Candidate block hash {} does not match given block hash {}",
                candidate_block_hash,
                block_hash
            ));
        }

        Ok(Self {
            block_hash,
            previous_block_hash,
            block_header_root,
            block_header_inclusion_proof,
            transactions_root,
            transactions_inclusion_proof,
            transaction_id,
            transaction_inclusion_proof,
            transition_id,
            transition_inclusion_proof,
            commitment,
        })
    }

    /// Returns the block hash used to prove inclusion of ledger-consumed records.
    pub fn block_hash(&self) -> N::BlockHash {
        self.block_hash
    }

    /// Returns the previous block hash.
    pub fn previous_block_hash(&self) -> N::BlockHash {
        self.previous_block_hash
    }

    /// Returns the block header root.
    pub fn block_header_root(&self) -> N::BlockHeaderRoot {
        self.block_header_root
    }

    /// Returns the block header inclusion proof.
    pub fn block_header_inclusion_proof(&self) -> &MerklePath<N::BlockHeaderRootParameters> {
        &self.block_header_inclusion_proof
    }

    /// Returns the transactions root.
    pub fn transactions_root(&self) -> N::TransactionsRoot {
        self.transactions_root
    }

    /// Returns the transactions inclusion proof.
    pub fn transactions_inclusion_proof(&self) -> &MerklePath<N::TransactionsRootParameters> {
        &self.transactions_inclusion_proof
    }

    /// Returns the transaction ID.
    pub fn transaction_id(&self) -> N::TransactionID {
        self.transaction_id
    }

    /// Returns the transaction inclusion proof.
    pub fn transaction_inclusion_proof(&self) -> &MerklePath<N::TransactionIDParameters> {
        &self.transaction_inclusion_proof
    }

    /// Returns the transition ID.
    pub fn transition_id(&self) -> N::TransitionID {
        self.transition_id
    }

    /// Returns the transition inclusion proof.
    pub fn transition_inclusion_proof(&self) -> &MerklePath<N::TransitionIDParameters> {
        &self.transition_inclusion_proof
    }

    /// Returns the commitment.
    pub fn commitment(&self) -> N::Commitment {
        self.commitment
    }
}

impl<N: Network> FromBytes for LedgerProof<N> {
    #[inline]
    fn read_le<R: Read>(mut reader: R) -> IoResult<Self> {
        let block_hash = FromBytes::read_le(&mut reader)?;
        let previous_block_hash = FromBytes::read_le(&mut reader)?;
        let block_header_root = FromBytes::read_le(&mut reader)?;
        let block_header_inclusion_proof = FromBytes::read_le(&mut reader)?;
        let transactions_root = FromBytes::read_le(&mut reader)?;
        let transactions_inclusion_proof = FromBytes::read_le(&mut reader)?;
        let transaction_id = FromBytes::read_le(&mut reader)?;
        let transaction_inclusion_proof = FromBytes::read_le(&mut reader)?;
        let transition_id = FromBytes::read_le(&mut reader)?;
        let transition_inclusion_proof = FromBytes::read_le(&mut reader)?;
        let commitment = FromBytes::read_le(&mut reader)?;

        Ok(Self::new(
            block_hash,
            previous_block_hash,
            block_header_root,
            block_header_inclusion_proof,
            transactions_root,
            transactions_inclusion_proof,
            transaction_id,
            transaction_inclusion_proof,
            transition_id,
            transition_inclusion_proof,
            commitment,
        )
        .expect("Failed to deserialize a ledger inclusion proof"))
    }
}

impl<N: Network> ToBytes for LedgerProof<N> {
    #[inline]
    fn write_le<W: Write>(&self, mut writer: W) -> IoResult<()> {
        self.block_hash.write_le(&mut writer)?;
        self.previous_block_hash.write_le(&mut writer)?;
        self.block_header_root.write_le(&mut writer)?;
        self.block_header_inclusion_proof.write_le(&mut writer)?;
        self.transactions_root.write_le(&mut writer)?;
        self.transactions_inclusion_proof.write_le(&mut writer)?;
        self.transaction_id.write_le(&mut writer)?;
        self.transaction_inclusion_proof.write_le(&mut writer)?;
        self.transition_id.write_le(&mut writer)?;
        self.transition_inclusion_proof.write_le(&mut writer)?;
        self.commitment.write_le(&mut writer)
    }
}

impl<N: Network> Default for LedgerProof<N> {
    fn default() -> Self {
        let empty_commitment = N::Commitment::default();

        let header_tree = MerkleTree::<N::BlockHeaderRootParameters>::new(
            Arc::new(N::block_header_root_parameters().clone()),
            &vec![empty_commitment; N::POSW_NUM_LEAVES],
        )
        .expect("Ledger proof failed to create default header tree");

        let previous_block_hash = N::BlockHash::default();
        let header_root = *header_tree.root();
        let header_inclusion_proof = header_tree
            .generate_proof(2, &empty_commitment)
            .expect("Ledger proof failed to create default header inclusion proof");

        let block_hash = N::block_hash_crh()
            .hash(
                &[
                    previous_block_hash
                        .to_bytes_le()
                        .expect("Ledger proof failed to convert previous block hash to bytes"),
                    header_root
                        .to_bytes_le()
                        .expect("Ledger proof failed to convert header root to bytes"),
                ]
                .concat(),
            )
            .expect("Ledger proof failed to compute block hash");

        Self {
            block_hash,
            previous_block_hash,
            block_header_root: header_root,
            block_header_inclusion_proof: header_inclusion_proof,
            transactions_root: Default::default(),
            transactions_inclusion_proof: MerklePath::default(),
            transaction_id: Default::default(),
            transaction_inclusion_proof: MerklePath::default(),
            transition_id: Default::default(),
            transition_inclusion_proof: MerklePath::default(),
            commitment: empty_commitment,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[rustfmt::skip]
    fn ledger_proof_new_test<N: Network>() -> Result<()> {
        let ledger = Ledger::<N>::new()?;
        assert_eq!(ledger.latest_block_height(), 0);
        assert_eq!(ledger.latest_block_transactions()?.len(), 1);

        let expected_block = ledger.latest_block()?;
        let coinbase_transaction = expected_block.to_coinbase_transaction()?;
        let expected_commitments = coinbase_transaction.commitments();

        // Create a ledger proof for one commitment.
        let ledger_proof = ledger.to_ledger_inclusion_proof(expected_commitments[0])?;
        assert_eq!(ledger_proof.block_hash, expected_block.block_hash());
        assert_eq!(ledger_proof.previous_block_hash, expected_block.previous_block_hash());
        assert_eq!(ledger_proof.block_header_root, expected_block.header().to_header_root()?);
        // assert_eq!(ledger_proof.commitments_root(), expected_block.header().commitments_root());
        // assert!(ledger_proof.commitment_inclusion_proofs[0].verify(&ledger_proof.commitments_root, &expected_commitments[0])?);
        // assert!(!ledger_proof.commitment_inclusion_proofs[0].verify(&ledger_proof.commitments_root, &expected_commitments[1])?);
        // assert!(!ledger_proof.commitment_inclusion_proofs[1].verify(&ledger_proof.commitments_root, &expected_commitments[0])?);
        // assert!(!ledger_proof.commitment_inclusion_proofs[1].verify(&ledger_proof.commitments_root, &expected_commitments[1])?);
        // assert_eq!(ledger_proof.commitments[0], expected_commitments[0]);
        // assert_eq!(ledger_proof.commitments[1], Default::default());

        Ok(())
    }

    #[test]
    fn test_new() {
        ledger_proof_new_test::<crate::testnet1::Testnet1>().unwrap();
        ledger_proof_new_test::<crate::testnet2::Testnet2>().unwrap();
    }
}
