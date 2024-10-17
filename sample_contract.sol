// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

contract ZKProverMarketplace {
    // Define the hardware types: GPU, FPGA, CPU
    enum HardwareType {
        GPU,
        FPGA,
        CPU
    }

    // Struct representing a prover in the marketplace
    struct Prover {
        HardwareType hardware;
        // TODO: Use euint from fhEVM (ZAMA)
        uint reputation;
        bool available;
    }

    // Struct representing a ZK proof request
    struct ZKProofRequest {
        HardwareType requiredHardware;
        uint price;
        address requester;
        bool completed;
    }

    // Mappings for provers and requests
    mapping(address => Prover) public provers;
    mapping(uint => ZKProofRequest) public requests;
    uint public requestCounter;

    // Event emitted when a new proof request is created
    event ProofRequestCreated(
        uint requestId,
        address indexed requester,
        HardwareType hardware,
        uint price
    );
    event ProofCompleted(uint requestId, address indexed prover);

    // Modifier to ensure only available provers can take requests
    modifier onlyAvailableProver() {
        require(provers[msg.sender].available == true, "Prover not available.");
        _;
    }

    // Modifier to ensure only the requester can complete the job
    modifier onlyRequester(uint requestId) {
        require(
            requests[requestId].requester == msg.sender,
            "Not the original requester."
        );
        _;
    }

    // Register a prover in the marketplace
    function registerProver(HardwareType hardware) public {
        provers[msg.sender] = Prover(hardware, 0, true); // New prover with initial reputation of 0
    }

    // Submit a new proof request
    function submitProofRequest(
        HardwareType requiredHardware,
        uint price
    ) public payable {
        require(msg.value == price, "Job must be pre-funded.");

        requestCounter++;
        requests[requestCounter] = ZKProofRequest({
            requiredHardware: requiredHardware,
            price: price,
            requester: msg.sender,
            completed: false
        });

        emit ProofRequestCreated(
            requestCounter,
            msg.sender,
            requiredHardware,
            price
        );
    }

    // A prover takes on a ZK proof job
    function takeProofJob(uint requestId) public onlyAvailableProver {
        ZKProofRequest memory zkRequest = requests[requestId];

        // Ensure the prover has the correct hardware
        require(
            provers[msg.sender].hardware == zkRequest.requiredHardware,
            "Hardware mismatch."
        );

        // Mark prover as unavailable
        provers[msg.sender].available = false;
    }

    // Complete the proof computation and transfer funds
    function completeProof(uint requestId) public onlyRequester(requestId) {
        ZKProofRequest storage zkRequest = requests[requestId];
        require(!zkRequest.completed, "Proof already completed.");

        // TODO: Send to a gateway that does off-chain computation

        zkRequest.completed = true;
        provers[msg.sender].reputation++; // Increase reputation for successful proof computation
        provers[msg.sender].available = true; // Mark prover as available again

        // Transfer payment to the prover
        payable(msg.sender).transfer(zkRequest.price);

        emit ProofCompleted(requestId, msg.sender);
    }
}
