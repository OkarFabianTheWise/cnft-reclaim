use crate::state::{TwapStorage, PriceObservation};
use solana_program::program_error::ProgramError;

pub fn calculate_twap(
    twap_storage: &TwapStorage,
    current_timestamp: i64,
    window_seconds: i64,
) -> Result<u64, ProgramError> {
    if twap_storage.observation_count < 2 {
        // If we have less than 2 observations, return the latest price
        let latest_index = if twap_storage.current_index == 0 {
            twap_storage.observation_count - 1
        } else {
            twap_storage.current_index - 1
        } as usize;
        
        return Ok(twap_storage.observations[latest_index].price);
    }

    let target_timestamp = current_timestamp - window_seconds;
    let mut time_weighted_sum: u128 = 0;
    let mut total_weight: i64 = 0;

    // Get valid observations sorted by timestamp
    let mut valid_observations = Vec::new();
    for i in 0..twap_storage.observation_count {
        valid_observations.push(twap_storage.observations[i as usize]);
    }
    valid_observations.sort_by_key(|obs| obs.timestamp);

    // Calculate time-weighted average
    for i in 0..(valid_observations.len() - 1) {
        let current_obs = &valid_observations[i];
        let next_obs = &valid_observations[i + 1];
        
        // Skip observations that are too old
        if next_obs.timestamp < target_timestamp {
            continue;
        }
        
        // Calculate the time weight for this observation
        let start_time = std::cmp::max(current_obs.timestamp, target_timestamp);
        let end_time = std::cmp::min(next_obs.timestamp, current_timestamp);
        
        if end_time > start_time {
            let weight = end_time - start_time;
            time_weighted_sum += current_obs.price as u128 * weight as u128;
            total_weight += weight;
        }
    }

    if total_weight == 0 {
        // If no valid time weights, return latest price
        let latest = valid_observations.last().unwrap();
        return Ok(latest.price);
    }

    let twap = (time_weighted_sum / total_weight as u128) as u64;
    Ok(twap)
}
