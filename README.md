# soroban-bag

## Preface

This is a pirate-themed tutorial for building an OHM fork on Stellar - not because anyone should actually build one, as their mechanisms are pretty unsustainable, but because OHM uses a fair amount of useful DeFi concepts which can be applied to other, more valuable applications. I also just thought coding one would be fun :)

**Click the picture for Mandatory Music For Readin This'ere Manuscript**\
[![Pirates](https://img.youtube.com/vi/meb_4cEriyw/0.jpg)](https://www.youtube.com/watch?v=meb_4cEriyw&list=PL64D371E54EB82C2C)

## Intro

Ahoy Matey! So you be wantin' ta assemble a crew of scallywags to journey the high seas? Well... I like the cut of your jib, sailor - could always do with a couple more sea dogs on the open ocean.

I suppose I could learn yarr a thin' r two about creatin' a decentralized finance protocol that there provides aspirin' gentlemen o' fortune with the three essential ingredients to inspire potential swashbucklers to set out on the 'igh seas o' the blockchain, treasure, adventure, an' skirmishes.

# Seven Seas Protocol Overview

This 'ere protocol, Seven Seas, 'as everythin' ye be lookin fer - we be fixin' to create DOUBLOON tokens fer treasure (every gentleman o' fortune likes a bloody doubloon), VOYAGES to let the treasure 'ungry sea dogs head adventuring with the promise o' doubloons at the end, an' most importantly - RAIDS ta' let the most bloodthirsty o' the bunch attack other voyages hopin' to get their 'ands on some plunder.

## Components

![Booty](/images/doubloons.jpg)

### Doubloons

_Doubloon_ tokens are the treasure of the Seven Seas protocol. Brave seafarers embark on _Voyages_ with the promise of a hefty reward of _Doubloons_ at the end. Once a scallywag gets his 'ands on a stash o _Doubloons_ they can either _Bury_ 'em, or they can spend em on _Raids_.

#### Implementation

We be usin' a [standard token contract](https://soroban.stellar.org/docs/built-in-contracts/token) fer _Doubloons_.

![Barbossa](/images/Barbossa.jpg)

### Captain

Just like a ship, the Seven Seas bay protocol needs a Cap'n ta direct er. The Cap'n be in charge o' the crucial tasks o' modifyin' the interest rate fer buried _Doubloon_ tokens, creatin' new _Voyage_ opportunities fer brave scallywags ta undertake, governin 'ow often we can expect gentlemen o' fortune ta _Raid_ each otha', an' decidin' what ta do with the Seven Seas treasury.

Yarr got a few options fer who to make tha Cap'n, each comes with it be own risks an' benefits:

1. Single Captain\
   Yarr can become a regularrr old pirate king an' install yerself as Cap'n, but yarr could make some scallywags mighty fearful o' usin' da Pirate Bay protocol, since they mightn't trust yarr all da way (who can blame 'em? ye be a regularrr scoundrel).
2. Multisig\
   Yarr can delegate the responsibility to a multisig account, thus creatin' a council o' pirate lords. This'ere option spreads out the trust, but we all know us gentlemen o' fortune don't always agree on a lot o' things, so decisions could be complicated.
3. Governance Contract\
   Yarr can engage in democracy an' set a governance contract as the captain, with buried DOUBLOON tokens bein' used ta vote on decisions. Unfortunately these ere contracts oftentimes work pretty slowly, but givin' decision makin' power to the common privateer be pretty wondrous.

Consider yer decision carefully as it'll influence the future o' this here 'ere enterprise. Ye wouldn't like to turn out as one o' the cursed protocols that there haunts these waters, doomed to wander aimlessly in Davy Jones' Locker fer eternity all because they 'ad a bad Cap'n.

#### Implementation

The _Captain_ is set during contract initialization and has access to the following functions

```rust
    /******** Captain only functions ********/
    #[doc = "
    Creates a new voyage offering
    - voyage_asset is the asset used to fund the voyage
    - price is the cost to embark on a voyage in voyage asset
    - available_voyages is the maximum number of voyages that can be embarked on for this voyage offering
    "]
    fn new_voyage(e: Env, voyage_asset: BytesN<32>, price: BigInt, available_voyages: BigInt);

    #[doc = "
    Transfers funds held in the contract
    - token_id is the address of the token being transferred
    - to is the destination for the transfer
    - amount is the amount of tokesn to transfer
    "]
    fn xfer_held(e: Env, token_id: BytesN<32>, to: Identifier, amount: BigInt);

    #[doc = "
    Sets the rebase rate for buried doubloons (the rate at which doubloons grow when buried)
    - rate is the per 100 block rebase rate for buried doubloons
    "]
    fn set_rate(e: Env, rate: BigInt);

    #[doc = "
    Sets a new captain for the Seven Seas protocol
    - new_captain is the address of the new captain
    "]
    fn set_capn(e: Env, new_captain: Identifier);

    #[doc = "
    Sets the target raid interval (how often raids should occur)
    - tgt_raid_int is the target number of blocks between raids
    "]
    fn set_tgt_ri(e: Env, tgt_raid_int: u32);
```

![Bury](/images/bury.jpg)

### Burying

E'ry swashbuckler worth is salt knows what ta do with some booty once ya get yer mits on it, yarr find a deserted island and _Bury_ it. _Burying_ yer _Doubloons_ lets ye earn interest based on rebase rate set by the _Captain_, paid in more _Doubloons_, so when ye return ta dig em up later yarr'll receive more _Doubloons_ than ye originally 'ad. Don't ask my why buryin' yarr treasure creates more treasure, yer readin' a tutorial written in piratespeak, we be clearly outa the realm o reason.

#### Implementation

The following functions are used to _Bury_ an' unearth _Doubloons_

```rust
    #[doc = "
    Buries doubloons
    - amount is the number of doubloons to be buried
    "]
    fn bury(e: Env, amount: BigInt);

    #[doc = "
    Unearths doubloons
    - amount is the number of doubloons to be unearthed
    "]
    fn unearth(e: Env, amount: BigInt);
```

We store buried tokens in the Seven Seas contract memory as it's simpler than creating another token to represent them. Also of note, we use an index to track how many _Doubloons_ a buried doubloon is worth. This index is a global value that grows based on the rate, and is updated every time any swashbuckler calls the `bury` or `unearth` function. The rate tracks buried doubloon interest on a per-100 blocks basis. Because of this, the index is scaled to twice the normal decimal amount to ensure that it can track buried doubloon value with high precision.

![Voyage](/images/voyage.jpg)

### Voyages

_Voyages_ be the main activity carried out by the buccaneers o' Seven Seas (besides a 'ealthy bit o drinkin' an fightin'). _Voyage_ opportunities are created by the _Captain_ whenever they please, each lasts seven days an' can be embarked upon at any time durin' that window o opportunity. 'owe'er, be warned, there are a limited number o' _Voyages_ available in every opportunity, so don't tarry ar yarr'll miss out on the plunder. Af'er the end a the 7 days all surviving voyages receive one _Doubloon_, that there payout may seem stingy, but there isn't a limit ta 'ow many voyages each individual gentleman o' fortune can set out on (besides the maximum voyages set by tha Cap'n).

In order to embark on a _Voyage_ the adventurin fool must provide down some token o' another kind in order ta hire a crew and vessel. Tha price o' the voyage an' the token demanded is set by the _Captain_. All proceeds from 'irin' crews an' vessels are stored in the Seven Seas treasury fer tha Cap'n ta distribute or manage.

#### Implementation

The Cap'n uses the following function to create new voyage offerings

```rust
    #[doc = "
    Creates a new voyage offering
    - voyage_asset is the asset used to fund the voyage
    - price is the cost to embark on a voyage in voyage asset
    - available_voyages is the maximum number of voyages that can be embarked on for this voyage offering
    "]
    fn new_voyage(e: Env, voyage_asset: BytesN<32>, price: BigInt, available_voyages: BigInt);
```

Common scallywags use the following functions to embark on new voyages and redeem successfully completed ones

```rust
    #[doc = "
    Fund and embark on a voyage
    - voyage_id is the id of the voyage to embark on
    - num_voyages is the number of voyages to embark on
    "]
    fn voyage(e: Env, voyage_id: i32, num_voyages: BigInt);

    #[doc = "
    End and redeem a voyage
    - voyage_id is the id of the voyage the user wants to end
    "]
    fn end_voyage(e: Env, voyage_id: i32);
```

![Skirmish](/images/skirmish.jpg)

### Raids

_Raids_ are fer the bravest and most risk 'ungry scallywags in Seven Seas. Ye can spend _Doubloons_ to raid ano'er seafarers _Voyages_, when you attempt that honorable task there's a chance o stealin' the poor bastard's plunder. You must always spend _Doubloons_ equal to 1/100th the promised payout from all the _Voyages_ the other seafarers embarked on for a given _Voyage_ offerin'. So if the voyager funded 1000 voyages raidin' em be fixin' ta cost ya 10 doubloons, an' if the raid be successful ye'll get all 1000 doubloons the voyager be promised. The unlucky target o a successful raid loses all their voyages.

All rumguzzlers know tha' launchin a successful raid takes careful preparation. The probability o' success for a Seven Seas raid ranges from `0-1.25%` based on 'ow long it's been since the last raid, an' the interval the _Captain_ set for raids. The formula for determinin' the probability for a raid's success is `min(blocks_since_last_raid/raid_interval,0.0125)`. This 'ere ensures that raids aren't too frequent to scare off voyagers, but are frequent enough to keep the 'igh seas excitin'.

#### Implementation

The raid function is still un-implemented pending the addition of PRNG, however, the `raid` function is my best guess at what it looks like when I peep at it through my spyglass

```rust
    #[doc = "
    Raid another users voyage
    - voyage_id is the id of the voyage the user wants to raid
    - user_id is the id of the user being raided
    - raider must have enough doubloons to pay for the raid, they need doubloons equal to 1/100th the number of voyages of the input type that the input user is on
    "]
    fn raid(e: Env, voyage_id: i32, user_id: Identifier);
```

You'll notice we publish events detailing the outcome of the raid, this is to allow users and contracts to keep track of whether or not a raid was successful.\
The probability check works by calculating a u32 based off of the probability for a successful raid, then generating a random u32 and seeing if it's smaller than the calculated u32.

![treasury](/images/something.webp)

### Treasury Operations

The _Captain_ is allowed to move the proceeds from voyage payments as they see fit. They could invest them in other DeFi protocols in the ocean o the blockchain, pay em out to _Doubloon_ 'olders, or any other action they wish. This be one o the reasons its important to pick a good Cap'n model.

#### Implementation

The _Captain_ uses the following function to move treasury funds

```rust
    #[doc = "
    Transfers funds held in the contract
    - token_id is the address of the token being transferred
    - to is the destination for the transfer
    - amount is the amount of tokesn to transfer
    "]
    fn xfer_held(e: Env, token_id: BytesN<32>, to: Identifier, amount: BigInt);
```

## Setup

Now we'll go over how to set up Seven Seas - no expedition can begin without a bit o' leg work an' provisionin'.

### Deploying The Contracts

First ya gotta build an deploy tha contract, I could go ahead and give detailed instructions on how ta do so ere but the voodoo docta's over at tha Stellar Development Foundation already did in the tutorials linked below.

_Building the contract_: https://soroban.stellar.org/docs/tutorials/build\
_Deploying the contract_: https://soroban.stellar.org/docs/tutorials/deploy-to-futurenet

Next ya need a DOUBLOON token firs'. Make sure ye set yerself as admin fer the tokens as ye'll 'ave to transfer ownership ta the Seven Seas protocol in a bit so it can move the tokens around.

If ye be just experimentin' with the protocol on yer own ye can deploy the tokens ta a local Futurenet instance usin' yer terminal an' the commands below.
Make sure ye be [connected to the futurenet first](https://soroban.stellar.org/docs/tutorials/deploy-to-futurenet).

Deploy DOUBLOON Token

```

soroban token create \
--name DOUBLOON \
--symbol DBLN \
--decimal 7 \
--secret-key <YOUR SECRET KEY> \
--rpc-url http://localhost:8000/soroban/rpc \
--network-passphrase 'Test SDF Future Network ; October 2022'

```

Once ye run this commands yer terminal will return yarr the contract address o the token.

After ye deploy this ere token contract and the Seven Seas contract you can go ahead and run the `set_admin` function on the DOUBLOON token contract and set the admin to the Seven Seas contract address. If yarr on a local instance o futurenet, yarr can run the function with the following terminal command

```

soroban invoke \
 --id <TOKEN CONTRACT ID> \
 --secret-key <YOUR SECRET KEY> \
 --rpc-url http://localhost:8000/soroban/rpc \
 --network-passphrase 'Test SDF Future Network ; October 2022' \
 --fn set_admin \
 --arg <SEVEN SEAS CONTRACT ID>

```

### Implementation

After ya deploy the Seven Seas protocol smart contract, The initialize function be the key to sending the Seven Seas protocol out inta the ocean of the blockchain.

```rust
    #[doc = "
    Initializes the contract
    - the caller will be set as the captain
    - the base_token_id is the address of the doubloon token
    - the rate is the per-100-block rebase rate for buried_doubloon tokens
    - the target_raid_interval is the goal number of blocks between raids
    "]
    fn initialize(e: Env, base_token_id: BytesN<32>, rate: BigInt, target_raid_interval: u32);

```

#### Inputs

Ye need to provide the function call with a few parameters that there the Seven Seas protocol be fixin' to need to survive the treacherous waters o the blockchain.

**Parameters:**

1. `base_token_id`: Here the contract be expectin' the DOUBLOON token contract id
2. `rate`: Now ye gotta provide a per-block rate fr buried DOUBLOONs to grow at
3. `target_raid_interval`: Ye remember 'ow we talked about raidin' an pilligin' earlier (in this here tutorial, not in the pub ye scallywag)? This here parameter sets the number o blocks that there should pass between each raid. This here ain't a 'ard limit, it be impossible to set rules around engagements between denizens o' the high seas, but we'll use some probability, an' the knowledge that pirates be greedy bastards who ne'er loved their mothers (although yer mother definitely been lovin' me lately if ye catch me drift) to encourage them to raid eacho'er with the frequency set by this here interval parameter

## Conclusion

That's it! Have a jolly ol time piratin' on that blockchain, ope ya learned a bit about trackin' interest rates, storin' balances, an doin' some probability junk.
