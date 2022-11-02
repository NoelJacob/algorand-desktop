# How It Works
This uses the Tauri rust framework. It uses the native webview on the device to render UI. It is miles better than Electron and alternatives. It's mobile development is in alpha stage.

It uses a standard SvelteKit source folder for the UI but contains backend in src-tauri folder. The src/routes/+pages.svelte has frontend code. The data is stored in .storage.dat file by Tauri persist plugin. This is a proof-of-concept and only use burner testnet account.

Finally, Tauri's app design aligns with Algorand's green ideals. [Every byte saved is a leaf on a tree that gets to grow](https://tauri.app/blog/2022/06/19/tauri-1-0#environment). 

![image](https://user-images.githubusercontent.com/54862022/199580542-65b0878b-c018-45ad-8777-da40cdf89a88.png)


# How To Run Dev

```
pnpm i
pnpm tauri dev
```
# How To Run Build
```
pnpm i
pnpm tauri build
```
Builds can be installed from `./src-tauri/target/release/bundle/`.

## Future
- Secure the storage. Currently it's unencrypted because password login on open isn't implimented.
- Mobile app builds
- Add mint facility
- More NFT info
- In app chain explorer

