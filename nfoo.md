a@abraham:~$ solana-keygen new
Generating a new keypair

For added security, enter a BIP39 passphrase

NOTE! This passphrase improves security of the recovery seed phrase NOT the
keypair file itself, which is stored as insecure plain text

BIP39 Passphrase (empty for none): 

Wrote new keypair to /home/a/.config/solana/id.json
=========================================================================
pubkey: CEF3tMXEGHeiLaySKiesydzXCjjanm93wE4NgnnBdEum
=========================================================================
Save this seed phrase and your BIP39 passphrase to recover your new keypair:
ticket cricket hello voice monster price dumb region wait cream soap sock
=========================================================================
a@abraham:~$ solana address
CEF3tMXEGHeiLaySKiesydzXCjjanm93wE4NgnnBdEum
a@abraham:~$ solana config set -ud
Config File: /home/a/.config/solana/cli/config.yml
RPC URL: https://api.devnet.solana.com 
WebSocket URL: wss://api.devnet.solana.com/ (computed)
Keypair Path: /home/a/.config/solana/id.json 
Commitment: confirmed 
a@abraham:~$ solana airdrop 2
Requesting airdrop of 2 SOL


