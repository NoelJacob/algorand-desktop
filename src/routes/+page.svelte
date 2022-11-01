<script>
    import logo from "$lib/assets/algo.svg"
    import reload from "$lib/assets/reload.svg"
    import Algonaut from '@thencc/algonautjs'
    import {Store} from 'tauri-plugin-store-api';
    import {CID} from 'multiformats/cid'
    import * as digest from 'multiformats/hashes/digest'
    import * as mfsha2 from 'multiformats/hashes/sha2'
    import error from "$lib/assets/error.svg"
    import {writeText} from '@tauri-apps/api/clipboard';

    const algonaut = new Algonaut({
        BASE_SERVER: 'https://testnet-algorand.api.purestake.io/ps2',
        INDEX_SERVER: 'https://testnet-algorand.api.purestake.io/idx2',
        LEDGER: 'TestNet',
        PORT: '',
        API_TOKEN: {'X-API-Key': 'B3SU4KcVKi94Jap2VXkK83xx38bsv95K5UZm2lab'}
    });
    const store = new Store('.settings.dat');

    let storedAcc = getStoredAccInfo(), balance = getBalance(), addr = getAddr(), recent = getRecentPayments(),
        asas = getAssests();

    function refresh() {
        storedAcc = getStoredAccInfo();
        balance = getBalance();
        recent = getRecentPayments();
        asas = getAssests();
        addr = getAddr();
    }

    async function getStoredAccInfo() {
        const mnemonic = await store.get('account');
        const account = algonaut.recoverAccount(mnemonic);
        algonaut.setAccount(account);
        if (mnemonic) {
            return algonaut.getAccountInfo(account.addr);
        }
    }

    async function getBalance() {
        const acc = await storedAcc;
        if (!acc)
            return Promise.reject();
        return algonaut.sdk.microalgosToAlgos(acc.amount);
    }

    async function getAddr() {
        const acc = await storedAcc;
        if (!acc)
            return Promise.reject();
        return acc.address;
    }

    async function getAssests() {
        const acc = await storedAcc;
        if (!acc)
            return Promise.reject();
        const x = acc.assets.map((asa) => algonaut.getAssetInfo(asa['asset-id']));
        const assets = await Promise.all(x)
        return assets.map((z) => {
            let {name, url, reserve} = z.params;
            let webUrl = resolveProtocol(url, reserve);
            return {name, webUrl}
        });
    }

    async function getRecentPayments() {
        const acc = await storedAcc;
        if (!acc)
            return Promise.reject();
        const paymentsInfo = await algonaut.indexerClient.lookupAccountTransactions(acc.address).txType("pay").do();
        return paymentsInfo.transactions.map(
            (tx) => {
                const me = tx.sender === acc.address ? 'sender' : 'receiver';
                const other = me === 'sender' ? tx['payment-transaction'].receiver : tx.sender;
                const symbol = me === 'sender' ? '🔼' : '🔽';
                let note = null
                if (tx.note) {
                    try {
                        const encoded = algonaut.fromBase64(tx.note);
                        note = JSON.stringify(encoded, undefined, 2);
                    } catch (e) {
                        console.log(e)
                    }
                }
                return {
                    id: tx.id,
                    fee: algonaut.sdk.microalgosToAlgos(tx.fee),
                    confirmed: tx['confirmed-round'],
                    amount: algonaut.sdk.microalgosToAlgos(tx['payment-transaction'].amount),
                    note,
                    me,
                    other,
                    symbol
                }
            })
    }

    function resolveProtocol(url, reserveAddr = '') {

        if (url.endsWith('#arc3'))
            url = url.slice(0, url.length - 'ARC3_URL_SUFFIX'.length)

        let chunks = url.split('://')
        // console.log('resolve protocol:', url)
        // console.log(chunks)
        // Check if prefix is template-ipfs and if {ipfscid:..} is where CID would normally be
        if (chunks[0] === 'template-ipfs' && chunks[1].startsWith('{ipfscid:')) {
            // Look for something like: template:ipfs://{ipfscid:1:raw:reserve:sha2-256} and parse into components
            chunks[0] = 'ipfs'
            const cidComponents = chunks[1].split(':')
            if (cidComponents.length !== 5) {
                // give up
                // console.log('unknown ipfscid format')
                return url
            }
            const [, cidVersion, cidCodec, asaField, cidHash] = cidComponents

            // const cidVersionInt = parseInt(cidVersion) as CIDVersion
            if (cidHash.split('}')[0] !== 'sha2-256') {
                // console.log('unsupported hash:', cidHash)
                return url
            }
            if (cidCodec !== 'raw' && cidCodec !== 'dag-pb') {
                // console.log('unsupported codec:', cidCodec)
                return url
            }
            if (asaField !== 'reserve') {
                // console.log('unsupported asa field:', asaField)
                return url
            }
            let cidCodecCode
            if (cidCodec === 'raw') {
                cidCodecCode = 0x55
            } else if (cidCodec === 'dag-pb') {
                cidCodecCode = 0x70
            }

            // get 32 bytes Uint8Array reserve address - treating it as 32-byte sha2-256 hash
            const addr = algonaut.sdk.decodeAddress(reserveAddr)
            const mhdigest = digest.create(mfsha2.sha256.code, addr.publicKey)

            const cid = CID.create(parseInt(cidVersion), cidCodecCode, mhdigest)
            // console.log('switching to id:', cid.toString())
            chunks[1] = cid.toString() + '/' + chunks[1].split('/').slice(1).join('/')
            // console.log('redirecting to ipfs:', chunks[1])
        }

        // No protocol specified, give up
        if (chunks.length < 2) return url

        //Switch on the protocol
        switch (chunks[0]) {
            case 'ipfs': //Its ipfs, use the configured gateway
                return 'https://nftstorage.link/ipfs/' + chunks[1]
            case 'https': //Its already http, just return it
                return url
            // TODO: Future options may include arweave or algorand
        }

        return url
    }

    let payDialog = false, receiveDialog = false, setupDialog = false;

    let pAddr, pAmt, pNote, pResult, sResult, sSecret;

    async function setupWallet() {
        await store.set('account', sSecret);
        refresh();
        const acc = await storedAcc;
        if (!acc)
            return Promise.reject();
        return acc.address;
    }

    async function pay() {
        const acc = await storedAcc;
        if (!acc)
            return Promise.reject("Set Account first");
        let status
        try {
            status = await algonaut.sendAlgo({to: pAddr, amount: algonaut.sdk.algosToMicroalgos(pAmt), note: pNote});
        } catch (e) {
            return Promise.reject("Failed")
        }
        console.log(status)
        return status
    }

    async function recieve() {
        const acc = await storedAcc;
        if (!acc)
            return Promise.reject("Set Account first");
        return acc.address
    }
</script>

{#if payDialog}
    <div class="dialog">
        <form on:click|preventDefault>
            <label>
                Address:
                <input bind:value={pAddr}>
            </label>
            <label>
                Amount:
                <input bind:value={pAmt}>
            </label>
            <label>
                Note:
                <input bind:value={pNote}>
            </label>
            {#if pResult}
                <div class="dialog-res">
                    {#await pay()}
                        Sending...
                    {:then status}
                        Confirmed: {status.txId}
                    {:catch error}
                        {error}
                    {/await}
                </div>
            {/if}
            <div class="dialog-btn">
                <button class="btn" on:click={()=>{pAddr='';pAmt='';pNote=''; payDialog = false}}>Close</button>
                <button class="btn" on:click={()=>{pResult = true}} value="default">Confirm</button>
            </div>
        </form>
    </div>
{:else if receiveDialog}
    <div class="dialog receive">
        {#await recieve()}
            Loading...
            <div class="dialog-btn">
                <button class="btn" on:click={()=>{receiveDialog = false}}>Close</button>
            </div>
        {:then addr}
            <div class="dialog-res">
                Your address: {addr}
            </div>
            <div class="dialog-btn">
                <button class="btn" on:click={()=>{receiveDialog = false}}>Close</button>
                <button class="btn" on:click={()=>{writeText(addr).then(()=>{receiveDialog = false})}} value="default">
                    Copy
                </button>
            </div>
        {:catch _}
            Add account to continue
            <div class="dialog-btn">
                <button class="btn" on:click={()=>{receiveDialog = false}}>Close</button>
            </div>
        {/await}
    </div>
{:else if setupDialog}
    <div class="dialog">
        <form on:click|preventDefault>
            <label>
                Secret Key:
                <input bind:value={sSecret}>
                Seperated by spaces
            </label>
            {#if sResult}
                <div class="dialog-res">
                    {#await setupWallet() then status}
                        Recovered {status}
                    {:catch error}
                        {error}
                    {/await}
                </div>
            {/if}
            <div class="dialog-btn">
                <button class="btn" on:click={()=>{setupDialog = false}}>Close</button>
                <button class="btn" on:click={()=>{sResult = true}} value="default">Recover</button>
            </div>
        </form>
    </div>
{/if}

<div class="dashboard">
    <div class="left">
        <div class="balance">
            {#await balance}
                <div class="bal">loading...</div>
            {:then a}
                <div class="bal">{a} <img alt="Algo coin" style="width: 2.4rem; height: 2.4rem;" src={logo}></div>
            {:catch _}
                <button class="setup" on:click={()=>{setupDialog=true}}>
                    Setup Wallet
                </button>
            {/await}
            <div class="divider"></div>
            <div class="info">
                <div class="addr">
                    {#await addr then a}
                        {a.slice(0, 10) + "..." + a.slice(-6)}
                    {:catch _}
                        Add to continue
                    {/await}
                </div>
                <button class="refresh" on:click={refresh}>
                    <img alt="refresh" src={reload}>
                </button>
            </div>
        </div>
        <div class="payment">
            <button class="btn" on:click={()=>{payDialog = true}}>Pay</button>
            |
            <button class="btn" on:click={()=>{receiveDialog = true}}>Receive</button>
        </div>
        <div class="recent">
            <h1>Recent Payments</h1>
            {#await recent then a}
                {#each a as {id, fee, confirmed, me, other, amount, symbol, note} (id)}
                    <div class="txn">
                        <div>{symbol} {amount} Algos</div>
                        <div class="details">
                            <span class="where">{me === "sender" ? "to" : "from"}</span>
                            <span class="other">{other.slice(0, 15) + "..." + other.slice(-6)}</span>
                            {#if note}
                                <div class="note">{note}</div>
                            {/if}
                        </div>
                    </div>
                {/each}
            {:catch _}
                <div class="txn">
                    <div class="id">Start making transactions</div>
                </div>
            {/await}
        </div>
    </div>
    <div class="right">
        <h1>Assets</h1>
        <div class="asset-list">
            {#await asas then a}
                {#each a as {name, webUrl}}
                    <div class="asset">
                        <img class="asset-display" alt="nft image" src={webUrl} on:error={()=>{webUrl = error}}>
                        <div class="name">{name}</div>
                    </div>
                {/each}
            {:catch _}
                <div class="asset">
                    <div class="id" style="white-space: nowrap;">Add NFTS and other assets</div>
                </div>
            {/await}
        </div>
    </div>
</div>

<style>
    .dialog {
        position: fixed;
        top: 0;
        left: 0;
        z-index: 1;
        width: 100vw;
        height: 100vh;
        background: rgba(28, 30, 31, 1);
        display: flex;
        justify-content: center;
        align-items: center;
    }

    .receive {
        flex-direction: column;
    }

    .dialog > form {
        display: flex;
        flex-direction: column;
        justify-content: center;
        align-items: center;
        flex-basis: 1rem;
    }

    .dialog-res {
        color: yellow;
        margin: 1rem 0 0 0;
        word-wrap: anywhere;
    }

    .dialog-btn {
        margin-top: 1.5rem;
    }

    .dialog-btn > button {
        margin: 0.5rem 0 0 0;
    }

    .dialog > form > label {
        margin: 0.5rem 0 0 0;
    }

    .btn {
        background-color: #396cd8;
        border-radius: 7px;
        border: white;
        width: 8rem;
        filter: drop-shadow(0 0 0.7rem rgba(44, 90, 178, 0.8));
        color: white;
        font-family: "JetBrains Mono", monospace;
        font-weight: 700;
    }

    .payment {
        height: 2.2rem;
        width: 20rem;
        margin: 1rem 0 0 2rem;
        display: flex;
        flex-direction: row;
        justify-content: space-evenly;
        align-items: center;
    }

    .asset-display {
        display: block;
        /*image;*/
        height: 90%;
        width: 100%;
        overflow: clip;
        border-radius: 0.4rem;
    }

    .name {
        margin: 0.4rem 0 0 0;
        overflow: hidden;
        white-space: nowrap;
        text-overflow: ellipsis;
        font-family: "JetBrains Mono", monospace;
        font-weight: 700;
        font-size: 1.05rem;
    }

    .asset {
        width: 8rem;
        height: 10rem;
        margin: 1rem;
        text-align: center;
    }

    .right {
        margin: 2rem 0 0 2rem;
    }

    .asset-list {
        display: flex;
        flex-direction: row;
        flex-wrap: wrap;
    }

    .dashboard {
        display: flex;
        flex-direction: row;
        /*height: 100%;*/
    }

    .note {
        color: greenyellow;
    }

    .details {
        padding: 0.3rem 0 0 0;
        font-size: 0.9rem;
    }

    .where {
        color: #999;
    }

    .txn {
        padding: 0.6rem 0 0.6rem 0;
    }

    .recent {
        margin: 1rem 0 0 2rem;
        width: 20rem;
    }

    h1 {
        text-shadow: 0 0 2px rgba(208, 208, 208, 0.4);
    }

    .refresh > img {
        width: 1.2em;
    }

    .refresh {
        background: none;
        border: none;
        outline: none;
        cursor: pointer;
    }

    .addr {
        width: 85%;
        overflow: hidden;
        white-space: nowrap;
        text-overflow: ellipsis;
        letter-spacing: 0;
        font-size: 1.1em;
    }

    .info {
        width: 80%;
        display: flex;
        flex-direction: row;
        justify-content: space-evenly;
    }

    .bal {
        font-size: 2.4rem;
        text-shadow: 0 0 2px rgba(208, 208, 208, 0.6);
        height: 50%;
        min-width: 70%;
        text-align: start;
        line-height: 1.05em;
        display: flex;
        align-items: end;
        overflow: hidden;
        white-space: nowrap;
        text-overflow: ellipsis;
        letter-spacing: -0.08em;
    }

    .left {
        display: flex;
        flex-direction: column;
        justify-content: start;
    }

    .divider {
        width: 70%;
        border-bottom: 1px solid rgba(252, 251, 251, 0.3);
    }

    .setup {
        font-size: 1.85rem;
        text-shadow: 0 0 2px rgba(208, 208, 208, 0.6);
        height: 50%;
        width: 70%;
        text-align: start;
        line-height: 1.0em;
        transform: translateY(0.2em);
        display: flex;
        align-items: end;
        border: none;
        color: white;
        background: none;
    }

    .setup:hover {
        text-decoration: underline;
        text-underline-style: dashed-heavy;

    }

    .balance {
        /*margin: auto;*/
        height: 7rem;
        width: 20rem;
        background-color: #396cd8;
        border-radius: 15px;
        margin: 2rem 0 0 2rem;
        font-family: "JetBrains Mono", monospace;
        font-weight: 500;
        letter-spacing: -1px;
        color: white;
        filter: drop-shadow(0 0 0.7rem rgba(44, 90, 178, 0.8));
        display: flex;
        flex-direction: column;
        justify-content: space-around;
        align-items: center;
    }
</style>
