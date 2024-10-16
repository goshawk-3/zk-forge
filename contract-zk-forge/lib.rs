#![cfg_attr(not(feature = "std"), no_std)]

use ink::prelude::*;
use ink::storage;

/*
Pseudo implementatoin of a marketplace as contract
*/

#[ink::contract]
mod zk_proof_marketplace {

    /// Struct to represent a Job submitted by a Requester.
    #[ink(storage)]
    pub struct ZKProofMarketplace {
        jobs: storage::Mapping<JobId, Job>,
        provers: storage::Mapping<AccountId, Prover>,
        matchmaker: Matchmaker,
        next_job_id: JobId,
    }

    type JobId = u64;

    /// Job structure
    #[derive(scale::Encode, scale::Decode, Clone, PartialEq, Eq, Debug)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub struct Job {
        pub job_id: JobId,
        pub requester: AccountId,
        pub price: Balance,
        pub proof_type: ProofType,
        pub status: JobStatus,
    }

    #[derive(scale::Encode, scale::Decode, Clone, PartialEq, Eq, Debug)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum JobStatus {
        Open,
        InProgress,
        Completed,
        Cancelled,
    }

    #[derive(scale::Encode, scale::Decode, Clone, PartialEq, Eq, Debug)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum ProofType {
        ZkSNARK,
        ZkSTARK,
        ZKML,
    }

    /// Prover structure
    #[derive(scale::Encode, scale::Decode, Clone, PartialEq, Eq, Debug)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub struct Prover {
        pub account_id: AccountId,
        pub reputation: u64,
        /// pub hardware: HardwareInfo,
        pub performance_rating: u64,
    }

    /// Matchmaker to pair jobs with provers
    pub struct Matchmaker;

    impl Matchmaker {
        /// Matches a job with the best available prover
        pub fn match_job_to_prover(job: &Job, provers: Vec<Prover>) -> Option<Prover> {
            // Matching logic based on price, performance, and hardware requirements
            let mut suitable_provers: Vec<Prover> = provers; // TODO:

            suitable_provers.sort_by(|a, b| b.performance_rating.cmp(&a.performance_rating));
            suitable_provers.into_iter().next() // Return the best match
        }
    }

    impl ZKProofMarketplace {
        /// Constructor to initialize the contract
        #[ink(constructor)]
        pub fn new() -> Self {
            Self {
                jobs: storage::Mapping::new(),
                provers: storage::Mapping::new(),
                matchmaker: Matchmaker,
                next_job_id: 0,
            }
        }

        /// Add a new job by a requester
        #[ink(message)]
        pub fn submit_job(&mut self, proof_type: ProofType, price: Balance) -> JobId {
            let requester = self.env().caller();
            let job_id = self.next_job_id;
            self.jobs.insert(
                &job_id,
                &Job {
                    job_id,
                    requester,
                    price,
                    proof_type,
                    status: JobStatus::Open,
                },
            );
            self.next_job_id += 1;
            job_id
        }

        /// Register a prover
        #[ink(message)]
        pub fn register_prover(&mut self, performance_rating: u64) -> bool {
            let account_id = self.env().caller();
            let prover = Prover {
                account_id,
                reputation: 0,
                performance_rating,
            };
            self.provers.insert(&account_id, &prover);
            true
        }

        /// Match a job to a prover and start processing
        #[ink(message)]
        pub fn match_and_start_job(&mut self, job_id: JobId) -> Option<AccountId> {
            let job = self.jobs.get(&job_id).unwrap();
            let mut available_provers = Vec::new();

            for (account_id, prover) in self.provers.iter() {
                available_provers.push(prover);
            }

            let matched_prover = self
                .matchmaker
                .match_job_to_prover(&job, available_provers)?;
            let prover_id = matched_prover.account_id;
            // Update the job status
            let mut job = self.jobs.get(&job_id).unwrap();
            job.status = JobStatus::InProgress;
            self.jobs.insert(&job_id, &job);
            Some(prover_id)
        }

        /// Complete a job and release funds to the prover
        #[ink(message, payable)]
        pub fn complete_job(&mut self, job_id: JobId) -> bool {
            let job = self.jobs.get(&job_id).unwrap();
            let prover_id = self.env().caller();

            assert!(job.status == JobStatus::InProgress);
            // Validate proof (assumed by external verifier or ZK Oracle)

            // Transfer funds to prover
            self.env().transfer(prover_id, job.price).unwrap();
            let mut job = self.jobs.get(&job_id).unwrap();
            job.status = JobStatus::Completed;
            self.jobs.insert(&job_id, &job);
            true
        }

        /// Update prover reputation (could be automated based on job completion rate)
        #[ink(message)]
        pub fn update_prover_reputation(&mut self, prover_id: AccountId, new_reputation: u64) {
            let mut prover = self.provers.get(&prover_id).unwrap();
            prover.reputation = new_reputation;
            self.provers.insert(&prover_id, &prover);
        }
    }
}
