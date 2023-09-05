pub mod pyth;
pub use pyth::*;
pub mod kraken;
pub use kraken::*;
use chrono::Utc;
use hex;
use serde::Deserialize;
use switchboard_evm::sdk::EVMFunctionRunner;
pub use switchboard_utils::reqwest;
use ethers::{
    prelude::{abigen, SignerMiddleware},
    providers::{Http, Provider},
    types::Address,
};

abigen!(
    Receiver,
    r#"[ function callback(uint256[], address[], bytes32[], bytes[]) ]"#,
);
static DEFAULT_URL: &str = "https://goerli-rollup.arbitrum.io/rpc";

#[tokio::main(worker_threads = 12)]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _ = 0;
    // --- Initialize clients ---
    let function_runner = EVMFunctionRunner::new()?;
    let receiver: Address = env!("EXAMPLE_PROGRAM").parse()?;
    let provider = Provider::<Http>::try_from(DEFAULT_URL)?;
    let signer = function_runner.enclave_wallet.clone();
    let client = SignerMiddleware::new_with_provider_chain(provider, signer).await?;
    let receiver_contract = Receiver::new(receiver, client.into());

    // --- Logic Below ---
    // DERIVE CUSTOM SWITCHBOARD PRICE
    let kraken_url = "https://api.kraken.com/0/public/Ticker?pair=BTCUSD";
    let kraken_spot: KrakenTickerResponse = reqwest::get(kraken_url).await?.json().await?;
    let (_, kraken_btc_usd) = kraken_spot.result.iter().next().unwrap();
    let mut kraken_price = kraken_btc_usd.price();
    kraken_price.rescale(8);
    let switchboard_prices = vec![
        kraken_price.mantissa().into(),
        // kraken_price.mantissa().into(),
    ];

    // CHAINLINK
    // https://docs.chain.link/data-feeds/price-feeds/addresses/?network=arbitrum
    let chainlink_price_ids = vec![
        "0xA39434A63A52E749F02807ae27335515BA4b07F7".parse()?, // BTC/USD
        // "0xD4a33860578De61DBAbDc8BFdb98FD742fA7028e".parse()?, // ETH/USD
    ];

    // PYTH
    // https://pyth.network/developers/price-feed-ids#pyth-evm-testnet
    let pyth_price_ids = vec![
        "0xf9c0172ba10dfa4d19088d94f5bf61d3b54d5bd7483a322a982e1373ee8ea31b", // BTC/USD
        // "0xca80ba6dc32e08d06f1aa886011eed1d77c77be9eb761cc10d72b7d0a2fd57a6", // ETH/USD
    ];
    let pyth_vaas = pyth::fetch_testnet_vaas(pyth_price_ids.clone()).await?;
    let fee = pyth::fetch_fee_for_vaas(&pyth_vaas).await?;
    let pyth_price_ids: Vec<[u8; 32]> = pyth_price_ids
        .iter()
        .map(|x| hex::decode(&x[2..]).unwrap().try_into().unwrap())
        .collect();
    // --- END LOGIC ---

    println!(
        "{}, {:#?}, {:?}, {:?}",
        kraken_price.clone(),
        chainlink_price_ids.clone(),
        pyth_price_ids.clone(),
        pyth_vaas.clone()
    );

    // --- Send the callback to the contract with Switchboard verification ---
    let callback = receiver_contract.callback(
        switchboard_prices,
        chainlink_price_ids,
        pyth_price_ids,
        pyth_vaas,
    );
    // let callback = callback.value(fee);
    let expiration = (Utc::now().timestamp() + 120).into();
    let gas_limit = 5_500_000.into();
    function_runner.emit(receiver, expiration, gas_limit, vec![callback])?;
    Ok(())
}

/// Run `cargo test -- --nocapture`
#[cfg(test)]
mod tests {
    use crate::pyth::*;
    use crate::*;

    #[tokio::test]
    async fn test() -> Result<(), Box<dyn std::error::Error>> {
        let kraken_url = "https://api.kraken.com/0/public/Ticker?pair=BTCUSD";
        let kraken_spot: KrakenTickerResponse = reqwest::get(kraken_url).await?.json().await?;
        let (_, kraken_btc_usd) = kraken_spot.result.iter().next().unwrap();
        let mut kraken_price = kraken_btc_usd.price();
        kraken_price.rescale(8);
        let switchboard_prices = vec![
            kraken_price.mantissa().into(),
            kraken_price.mantissa().into(),
        ];
        // https://docs.chain.link/data-feeds/price-feeds/addresses/?network=arbitrum
        let chainlink_price_ids = vec![
            "0xA39434A63A52E749F02807ae27335515BA4b07F7".parse()?, // BTC/USD
            "0xD4a33860578De61DBAbDc8BFdb98FD742fA7028e".parse()?, // ETH/USD
        ];
        // https://pyth.network/developers/price-feed-ids#pyth-evm-testnet
        //
        let pyth_price_ids = vec![
            "0xf9c0172ba10dfa4d19088d94f5bf61d3b54d5bd7483a322a982e1373ee8ea31b", // BTC/USD
            "0xca80ba6dc32e08d06f1aa886011eed1d77c77be9eb761cc10d72b7d0a2fd57a6", // ETH/USD
        ];
        let pyth_vaas = pyth::fetch_testnet_vaas(pyth_price_ids.clone()).await?;
        // let _fee = pyth::fetch_fee_for_vaas(&pyth_vaas).await?;
        let pyth_price_ids: Vec<[u8; 32]> = pyth_price_ids
            .iter()
            .map(|x| hex::decode(&x[2..]).unwrap().try_into().unwrap())
            .collect();
        println!(
            "{}, {:#?}, {:?}, {:?}",
            kraken_price.clone(),
            chainlink_price_ids.clone(),
            pyth_price_ids.clone(),
            pyth_vaas.clone()
        );

        let function_runner = EVMFunctionRunner::new()?;
        let receiver: Address = env!("EXAMPLE_PROGRAM").parse()?;
        let provider = Provider::<Http>::try_from(DEFAULT_URL)?;
        let signer = function_runner.enclave_wallet.clone();
        let client = SignerMiddleware::new_with_provider_chain(provider, signer).await?;
        let receiver_contract = Receiver::new(receiver, client.into());
        let _callback = receiver_contract.callback(
            switchboard_prices,
            chainlink_price_ids,
            pyth_price_ids,
            pyth_vaas,
        );
        Ok(())
    }
}
