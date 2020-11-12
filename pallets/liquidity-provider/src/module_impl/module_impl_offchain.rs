use crate::{Balance, Call, Module, NextUnsignedAt, Trait};
use alloc::{string::String, vec::Vec};
use core::str;
use frame_support::storage::StorageValue;
use frame_system::offchain::{SendUnsignedTransaction, SignedPayload, Signer, SigningTypes};
use sp_runtime::offchain::{http, Duration};
use valiu_node_commons::{Asset, Collateral, Pair, PairPrice};

const ALLOWED_PAIRS: [Pair; 3] = [
    Pair::new(Asset::Btc, Asset::Collateral(Collateral::Usd)),
    Pair::new(Asset::Btc, Asset::Ves),
    Pair::new(Asset::Collateral(Collateral::Usd), Asset::Cop),
];

#[derive(
    Debug, Eq, Ord, parity_scale_codec::Decode, parity_scale_codec::Encode, PartialEq, PartialOrd,
)]
pub struct OffchainPairPricesPayload<N, P> {
    pub(crate) pair_prices: Vec<PairPrice<N>>,
    pub(crate) public: P,
}

impl<N, T> SignedPayload<T> for OffchainPairPricesPayload<N, T::Public>
where
    N: parity_scale_codec::Encode,
    T: SigningTypes,
{
    fn public(&self) -> T::Public {
        self.public.clone()
    }
}

impl<T> Module<T>
where
    T: Trait,
{
    pub(crate) fn fetch_pair_prices() -> Result<Vec<PairPrice<Balance<T>>>, http::Error> {
        let mut pair_prices = Vec::new();
        pair_prices.push(Self::fetch_and_parse_btc_usd()?);
        pair_prices.push(Self::fetch_and_parse_usd_cop()?);
        Ok(pair_prices)
    }

    pub(crate) fn fetch_pair_prices_and_submit_tx(
        block_number: T::BlockNumber,
    ) -> Result<(), &'static str> {
        let next_unsigned_at = <NextUnsignedAt<T>>::get();
        if next_unsigned_at > block_number {
            return Err("Too early to send unsigned transaction");
        }

        let pair_prices = Self::fetch_pair_prices().map_err(|_| "Failed to fetch price")?;

        let (_, result) = Signer::<T, T::OffchainAuthority>::any_account()
            .send_unsigned_transaction(
                |account| OffchainPairPricesPayload {
                    pair_prices: pair_prices.clone(),
                    public: account.public.clone(),
                },
                |payload, signature| Call::submit_pair_prices(payload.pair_prices, signature),
            )
            .ok_or("No local accounts accounts available.")?;
        result.map_err(|()| "Unable to submit transaction")?;
        Ok(())
    }

    pub(crate) fn incoming_pair_prices_are_valid(pair_prices: &[PairPrice<Balance<T>>]) -> bool {
        if ALLOWED_PAIRS.len() != pair_prices.len() {
            return false;
        }
        for allowed in ALLOWED_PAIRS.iter() {
            if !pair_prices.iter().any(|el| el.pair() == allowed) {
                return false;
            }
        }
        true
    }

    fn fetch_and_parse_btc_usd() -> Result<PairPrice<Balance<T>>, http::Error> {
        let res = Self::request_response(
            "https://min-api.cryptocompare.com/data/pricemulti?fsyms=BTC,USD&tsyms=BTC,USD",
        )?;
        Self::parse_btc_usd(&res).ok_or(http::Error::Unknown)
    }

    fn fetch_and_parse_usd_cop() -> Result<PairPrice<Balance<T>>, http::Error> {
        let res = Self::request_response("https://www.trmhoy.co/")?;
        Self::parse_usd_cop(&res).ok_or(http::Error::Unknown)
    }

    fn request_response(url: &str) -> Result<String, http::Error> {
        let deadline = sp_io::offchain::timestamp().add(Duration::from_millis(2_000));

        let request = http::Request::get(url);

        let pending = request
            .deadline(deadline)
            .send()
            .map_err(|_| http::Error::IoError)?;

        let response = pending
            .try_wait(deadline)
            .map_err(|_| http::Error::DeadlineReached)??;

        if response.code != 200 {
            return Err(http::Error::Unknown);
        }

        let body = response.body().collect::<Vec<u8>>();

        let body_string = String::from_utf8(body).map_err(|_| http::Error::Unknown)?;

        Ok(body_string)
    }
}
