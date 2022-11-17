# soroban-bag

## Preface

This is a pirate-themed tutorial for building an OHM fork on Stellar, not because anyone should actually build one, their mechanisms are pretty unsustainable. but because OHM uses a fair amount of useful DeFi concepts that can be applied to other, more valuable applications. I also just thought coding one would be fun :)

**Click 'ere for Mandatory Music For Readin This'ere Manuscript**\
[![Pirates](https://img.youtube.com/vi/meb_4cEriyw/0.jpg)](https://www.youtube.com/watch?v=meb_4cEriyw&list=PL64D371E54EB82C2C)

## Intro

Ahoy Maty! So you be wantin' ta attract a crew of scalleywags to journey the high seas with. Well that suits me - could always do with a couple more sea dogs on the open ocean.

I suppose I could learn yarr a thin' r two about creatin' a decentralized finance protocol that there provides aspirin' gentlemen o' fortune with the three essential ingredients to inspire potential swashbucklers to set out on the 'igh seas o' the blockchain, treasure, adventure, an' skirmishes.

# Pirates Bay Protocol Overview

This here 'ere protocol, Pirates Bay, 'as everythin' ye be lookin fer - we be fixin' to create DOUBLOON tokens fer treasure (every gentleman o' fortune likes a jolly doubloon), VOYAGES to let the treasure 'ungry sea dogs head adventuring with the promise o' doubloons at the end, an' most importantly - RAIDS ta' let the most bloodthirsty o' the bunch attack other voyages 'opin' to get their 'ands on some plunder.

## Components

![Booty](/images/doubloons.jpg)

### Doubloons

_Doubloon_ tokens are the treasure of the Pirates Bay protocol. Brave seafarers embark on _Voyages_ with the promise of a hefty reward of _Doubloons_ at the end. Once a scalleywag gets his 'ands on a stash o _Doubloons_ they can either _Bury_ em, or ya can spend em on _Raids_.

#### Implementation

We be usin' a [standard token contract](https://soroban.stellar.org/docs/built-in-contracts/token) fr _Doubloons_.

![Barbossa](/images/Barbossa.jpg)

### Captain

Just like a ship, the Pirates Bay bay protocol needs a Cap'n ta direct er. The Cap'n be in charge o' the crucial tasks o' modifyin' the interest rate fer buried _Doubloon_ tokens, creatin' new _Voyage_ opportunities fer brave scalleywags ta undertake, governin 'ow often we can expect gentlemen o' fortune ta _Raid_ eachotha, an' decidin what ta do with the Pirates Bay treasury.

Yarr got a few options fer who to make tha Cap'n, each comes with it be own risks an' benefits:

1. Single Captain\
   Yarr can become a regularrr old pirate king an' install yerself as Cap'n, but yarr could make some scalleywags mighty fearful o usin' da Pirate Bay protocol, since they mightnt trust yarr all da way (who can blame em? ye be a regularrr scoundral).
2. Multisig\
   Yarr can delegate the responsibility to a multisig account, thus creatin a council o pirate lords. This'ere option spreads out the trust, but we all know us gentlemen o' fortune don't always agree on a lot o' things, so decisions could be complicated.
3. Governance Contract\
   Yarr can engage in democracy an' set a governance contract as the captain, with buried DOUBLOON tokens bein used ta vote on decisions. Unfortanetly these ere contracts oftentimes work pretty slowley, but givin' decision makin' power to the common deckboy be pretty wonderous.

Consider yer decision carefully as it'll influence the future o this here 'ere enterprise. Ye wouldn't like to turn out as one o' the cursed protocols that there 'aunts these waters, doomed to wander aimlessly fer eternity all because they 'ad a bad Cap'n.

#### Implementation

The _Captain_ is set during contract initialization and has access to the following functions

```
    /******** Admin functions ********/
    fn new_voyage(e: Env, vyg_asset: BytesN<32>, price: BigInt, max_vygs: BigInt) {
        check_captain(&e, &Signature::Invoker);
        let voyage_id = get_last_voyage_id(&e) + 1;
        let voyage_info = VoyageInfo {
            vyg_asset,
            price,
            max_vygs,
            n_embarked: BigInt::zero(&e),
            expiration: e.ledger().sequence() + WEEK_IN_BLOCKS,
        };
        set_voyage(&e, voyage_id, voyage_info);
        set_last_voyage_id(&e, voyage_id);
    }

    // transfers contract holdings
    fn xfer_held(e: Env, token_id: BytesN<32>, to: Identifier, amount: BigInt) {
        //check that invoker is admin
        check_captain(&e, &Signature::Invoker);
        transfer(&e, token_id, to, amount);
    }

    fn set_rate(e: Env, rate: BigInt) {
        //check that invoker is admin
        check_captain(&e, &Signature::Invoker);
        let new_index = get_new_index(&e);
        set_index(&e, new_index);
        set_last_block(&e);
        set_rate(&e, rate);
    }

    fn set_tgt_ri(e: Env, interval: u32) {
        //check that invoker is admin
        check_captain(&e, &Signature::Invoker);
        set_target_raid_interval(&e, interval);
    }

    fn set_capn(e: Env, new_admin: Identifier) {
        //check that invoker is admin
        check_captain(&e, &Signature::Invoker);
        write_captain(&e, new_admin);
    }
```

![Bury](/images/bury.jpg)

### Burying

E'ry swashbuckler worth is salt knows what ta do with some booty once ya get yer mits on it, yarr find a deserted island and _Bury_ it. _Burying_ yer _Doubloons_ lets ye earn interest based on rebase rate set by the _Captain_, paid in more _Doubloons_, so when ye return ta dig em up later yarr'll receive more _Doubloons_ than ye originally 'ad. Don't ask my why buryin' yarr treasure creates more treasure, yer readin a tutorial written in piratespeak, we be clearly outa the realm o reason.

#### Implementation

The following functions are used to _Bury_ an' unearth _Doubloons_

```

    fn bury(e: Env, amount: BigInt) {
        let user_id = Identifier::from(e.invoker());
        let new_index = get_new_index(&e);
        set_index(&e, new_index.clone());
        set_last_block(&e);
        burn_token(&e, user_id.clone(), amount.clone());
        let bury_amount = amount * BigInt::from_i64(&e, SCALER * SCALER) / new_index.clone();
        add_buried(&e, user_id, bury_amount);
    }

    fn unearth(e: Env, amount: BigInt) {
        let user = Identifier::from(e.invoker());
        let new_index = get_new_index(&e);
        set_index(&e, new_index.clone());
        set_last_block(&e);
        subtract_buried(&e, user.clone(), amount.clone());
        let mint_amount = amount * new_index / BigInt::from_i64(&e, SCALER * SCALER);
        mint_token(&e, user, mint_amount);
    }
```

We store buried tokens in the Pirates Bay contract memory as it's simpler than creating another token to represent them. Also of note, we use an index to track how many _Doubloons_ a buried doubloon is worth. This index is a global value that grows based on the rate, and is updated every time any swashbuckler calls the `bury` or `unearth` function. The rate tracks buried doubloon interest on a per-100 blocks basis. Because of this, the index is scaled to twice the normal decimal amount to ensure that it can track buried doubloon value with high precision.

![Voyage](/images/voyage.jpg)

### Voyages

_Voyages_ be the main activity carried out by the buccaneers o' Pirates Bay (besides a 'ealth bit o drinkin' an fightin'). _Voyage_ opportunities are created by the _Captain_ whenever they please, each lasts 7 days an' can be embarked upon at any time durin' that window o opportunity. 'owe'er, be warned, there are a limited number o' _Voyages_ available in every opportunity, so don't tarry ar yarr'll miss out on the plunder. Af'er the end a the 7 days all surviving voyages receive one _Doubloon_, that there payout may seem stingy, but there isn't a limit ta 'ow many voyages each individual gentleman o' fortune can set out on (besides the maximum voyages set by tha Cap'n).

In order to embark on a _Voyage_ the adventurin fool must provide down some token o' another kind in order ta hire a crew and vessel. Tha price o' the voyage an' the token demanded is set by the _Captain_. All proceeds from 'irin' crews an' vessels are stored in the Pirates Bay treasury fer tha Cap'n ta distribute or manage.

#### Implementation

The Cap'n uses the following function to create new voyage offerings

```

    fn new_voyage(e: Env, vyg_asset: BytesN<32>, price: BigInt, max_vygs: BigInt) {
        check_captain(&e, &Signature::Invoker);
        let voyage_id = get_last_voyage_id(&e) + 1;
        let voyage_info = VoyageInfo {
            vyg_asset,
            price,
            max_vygs,
            n_embarked: BigInt::zero(&e),
            expiration: e.ledger().sequence() + WEEK_IN_BLOCKS,
        };
        set_voyage(&e, voyage_id, voyage_info);
        set_last_voyage_id(&e, voyage_id);
    }
```

Common scallywags use the following functions to embark on new voyages and redeem successfully completed ones

```
    /// Requires approval for `transfer_from` before running
    fn voyage(e: Env, voyage_id: i32, num_voyages: BigInt) {
        let mut voyage_info = get_voyage(&e, voyage_id);

        //check that the voyage is still available
        if voyage_info.expiration.clone() < e.ledger().sequence().into() {
            panic!("voyage no longer available");
        }
        if voyage_info.max_vygs.clone() - voyage_info.n_embarked.clone() < num_voyages.clone() {
            panic!("not enough voyage available");
        }
        let transfer_amount = voyage_info.price.clone() * num_voyages.clone();
        let user_id = Identifier::from(e.invoker());

        let voyage_asset_client = crate::token::Client::new(&e, voyage_info.vyg_asset.clone());

        voyage_asset_client.xfer_from(
            &Signature::Invoker,
            &BigInt::zero(&e),
            &user_id,
            &get_contract_id(&e),
            &transfer_amount,
        );

        set_user_voyage(&e, user_id, voyage_id, num_voyages.clone());

        //update voyage info
        voyage_info.n_embarked = voyage_info.n_embarked.clone() + num_voyages.clone();

        set_voyage(&e, voyage_id, voyage_info);
    }

    fn end_voyage(e: Env, voyage_id: i32) {
        let user_id = Identifier::from(e.invoker());
        let user_voyage_amt = get_user_voyage(&e, user_id.clone(), voyage_id);
        // NOTE: this may not be necessary, could just let contract panic from null result, but think this is clearer
        if user_voyage_amt == BigInt::zero(&e) {
            panic!("user has no voyages for this voyage id");
        }
        // user gets shells equal to the number of voyages they finished
        mint_token(&e, user_id.clone(), user_voyage_amt);
        remove_user_voyage(&e, user_id, voyage_id);
    }
```

![Skirmish](/images/skirmish.jpg)

### Raids

_Raids_ are fer the bravest and most risk 'ungry scallywags in Pirates Bay. Ye can spend _Doubloons_ to raid another seafarers _Voyages_, when you do so there's a chance o stealin the poor bastards plunder. You must always spend _Doubloons_ equal to 1/100th the promised payout from all the _Voyages_ the other seafarers embarked on for a given _Voyage_ offerin'. So if the voyager funded 1000 voyages raidin' em be fixin' ta cost ya 10 doubloons, an' if the raid be successful ye'll get all 1000 doubloons the voyager be promised. The unlucky target o a successful raid loses all their voyages.

All rumguzzlers know tha launchin a successful raid takes careful preparation. The probability o success for a Pirates Bay raid ranges from `0-1.25%` based on 'ow long it's been since the last raid, an' the interval the _Captain_ set for raids. The formula for determinin the probability for a raid's success is `min(blocks_since_last_raid/raid_interval,0.0125). This 'ere ensures that raids aren't too frequent to scare off voyagers, but are frequent enough to keep the 'igh seas excitin'.

#### Implementation

The raid function is still un-implemented pending the addition of PRNG, however, this is my best guess at what it'll look like

```
    fn raid(e: Env, voyage_id: i32, voyager_id: Identifier) {
        // waiting on PRNG pull request https://github.com/stellar/rs-soroban-env/pull/544
        panic!("Not Implemented");

        let user_voyage_amt = get_user_voyage(&e, voyager_id.clone(), voyage_id);
        let raider_id = Identifier::from(e.invoker());
        // NOTE: this may not be necessary, could just let contract panic from null result, but think this is clearer
        if user_voyage_amt == BigInt::zero(&e) {
            panic!("user has no voyages for this voyage id");
        }
        // calculate the amount of shells required to perform the raid
        let raid_cost = user_voyage_amt / BigInt::from_i64(&e, 100);
        // burn the shells
        burn_token(&e, raider_id, raid_cost);

        // calculate probability of raid - scaled by how long it has been since the last raid - we target 1 raid per 600 blocks - probability cant be greater than 1.25%
        // scale by 10000 to avoid floating point math
        let current_block: u32 = e.ledger().sequence().into();
        let mut raid_probability: u32 =
            (current_block - get_last_raid_block(&e)) * 10000 / get_target_raid_interval(&e);
        if raid_probability > 12500 {
            raid_probability = 12500;
        }
        // calculate the max random number for a successful raid based on raid probability
        let max_ok_PRNG = MAX / 1000000 * raid_probability;
        // check if the raid was successful
        // TODO: Use PRNG to determine if raid was successful, waiting on this PR https://github.com/stellar/rs-soroban-env/pull/544
        let prng_u32: u32 = e.prng_next_u32(RawVal::from_bool(true));
        if prng_u32 < max_ok_PRNG {
            // raid was successful, user loses all their voyages, raider gets shells
            remove_user_voyage(&e, voyager_id, voyage_id);
            mint_token(&e, raider_id.clone(), user_voyage_amt);
            e.events().publish(
                (symbol!("raid_won"), voyage_id, voyager_id, current_block),
                true,
            );
        } else {
            e.events().publish(
                (symbol!("raid_won"), voyage_id, voyager_id, current_block),
                false,
            );
        }
        //update last raid block
        set_last_raid(&e);
    }
```

You'll notice we publish events detailing the outcome of the raid, this is to allow users and contracts to keep track of whether or not a raid was successful.\
The probability check works by calculating a u32 based off of the probability for a successful raid, then generating a random u32 and seeing if it's smaller than the calculated u32.

![treasury](/images/something.webp)

### Treasury Operations

The _Captain_ is allowed to move the proceeds from voyage payments as they see fit. They could invest them in other DeFi protocols in the ocean o the blockchain, pay em out to _Doubloon_ 'olders, or any other action they wish. This be one o the reasons its important to pick a good Cap'n model.

#### Implementation

The _Captain_ uses the following function to move treasury funds

```

    // transfers contract holdings
    fn xfer_held(e: Env, token_id: BytesN<32>, to: Identifier, amount: BigInt) {
        //check that invoker is admin
        check_captain(&e, &Signature::Invoker);
        transfer(&e, token_id, to, amount);
    }
```

## Setup

Now we'll go over how to set up Pirates Bay - no expedition can begin without a bit o' leg work an' provisionin'.

### Deploying The Contracts

First ya gotta build an deploy tha contract, I could go ahead and give detailed instructions on how ta do so ere but the voodoo docta's over at tha Stellar Development Foundation already did in the tutorials linked below.

_Building the contract_: https://soroban.stellar.org/docs/tutorials/build\
_Deploying the contract_: https://soroban.stellar.org/docs/tutorials/deploy-to-futurenet

Next ya need a DOUBLOON token firs'. Make sure ye set yerself as admin fer the tokens as ye'll 'ave to transfer ownership ta the Pirates Bay protocol in a bit so it can move the tokens around.

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

After ye deploy this ere token contract and the Pirates Bay contract you can go ahead and run the `set_admin` function on the DOUBLOON token contract and set the admin to the Pirates Bay contract address. If yarr on a local instance o futurenet, yarr can run the function with the following terminal command

```

soroban invoke \
 --id <TOKEN CONTRACT ID> \
 --secret-key <YOUR SECRET KEY> \
 --rpc-url http://localhost:8000/soroban/rpc \
 --network-passphrase 'Test SDF Future Network ; October 2022' \
 --fn set_admin \
 --arg <Pirates Bay CONTRACT ID>

```

### Implementation

After ya deploy the Pirates Bay protocol smart contract, The initialize function be the key to sending the Pirates Bay protocol out inta the ocean of the blockchain.

```

    fn initialize(e: Env, base_token_id: BytesN<32>, rate: BigInt, target_raid_interval: u32) {
        if e.data().has(DataKey::BaseToken) {
            panic!("contract already initialized");
        }
        //check if PiratesBay contract is the admin for base tokens and share token
        /*** Note - currently not possible as you can't read token admins TODO: file issue */

        set_base_token(&e, base_token_id);
        set_total_buried(&e, BigInt::zero(&e));
        set_rate(&e, rate);
        set_index(&e, BigInt::from_i64(&e, SCALER * SCALER));
        set_last_block(&e);
        set_decimals(&e);
        set_target_raid_interval(&e, target_raid_interval);
        write_captain(&e, Identifier::from(e.invoker()));
    }

```

#### Inputs

Ye need to provide the function call with a few parameters that there the Pirates Bay protocol be fixin' to need to survive the treacherous waters o the blockchain.

**Parameters:**

1. `base_token_id`: Here the contract be expectin the DOUBLOON token contract id
2. `rate`: Now ye gotta provide a per-block rate fr burried DOUBLOONs to grow at
3. `target_raid_interval`: Ye remember 'ow we talked about raidin an pilligin earlier (in this here tutorial, not in the pub ye scallywag)? This here parameter sets the number o blocks that there should pass between each raid. This here ain't a 'ard limit, it be impossible to set rules around engagements between denizens o' the high seas, but we'll use some probability, an' the knowledge that pirates be greedy bastards who ne'er loved their mothers (although yer mother definitely been lovin me lately if ye catch me drift) to encourage them to raid eachother with the frequency set by this here interval parameter

## Conclusion

That's it! Have a jolly ol time piratin on that blockchain, ope ya learned a bit about trackin interest rates, storin balances, an doin some probability junk.
