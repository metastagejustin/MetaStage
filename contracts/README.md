On a high level the contract API boils down to the following: 

1. Check the epoch timeline of creator registration, followed by posterior funding by users. More explicitly, the contract
operates per `Epoch`. Each time a new epoch starts, there is a period of creator registration, where creators can register
their work/projects, and establish all the metadata related to this. During this period, all the data necessary for NFT minting
is provided. After creator registration ends, the funding period starts. In this case, users are incentivized to come to the
platform and fund the projects they consider to have more potential. The epoch changing and inner-epoch dynamics is orchestrated
by an admin. This is the sole role of the admin in the entire protocol.

2. Creator registration process. The creator registration process is done via the platform by means of an RPC call to the contract.
The relevant method is `creator_registration`. It receives as input a `Metadata` type. Notice, that each project has `3` tiers of NFTs.
Each tier represents the scarcity of the NFT associated to the project (namely `Common`, `Uncommon`, `Rare`)
 Internally it consists of the following parameters

 - An array of size `3` of type `u128` (big integer) with the price of each NFT, for each tier;
 - An array of size `3` of type `String` with the Token/Project title, for each tier;
 - An array of size `3` of type `String` with the Token/Project description, for each tier;
 - An array of size `3` of type `String` with the Token/Project media link (either IPFS, or own cloud storage), for each tier;
 - An array of size `3` of type `u32` (integer) with the number of copies for each NFT, for each tier;
 - An array of size `3` of type `String` with extra metadata associated with the Creator project, for each tier;
 - An array of size `3` of type `String` with references associated with the Creator project, for each tier.

The above input data, must be retrieved directly from the Creator, via the platform. 

3. 
