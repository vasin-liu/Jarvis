use std::collections::HashMap;

use store::ChunkHit;

/// Reciprocal Rank Fusion across ranked hit lists (higher score = better).
pub fn reciprocal_rank_fusion(lists: &[&[ChunkHit]], rrf_k: f64, final_k: usize) -> Vec<ChunkHit> {
    let mut scores: HashMap<i64, f64> = HashMap::new();
    let mut by_id: HashMap<i64, ChunkHit> = HashMap::new();

    for list in lists {
        for (rank, hit) in list.iter().enumerate() {
            let contribution = 1.0 / (rrf_k + rank as f64 + 1.0);
            *scores.entry(hit.chunk_id).or_default() += contribution;
            by_id.entry(hit.chunk_id).or_insert_with(|| hit.clone());
        }
    }

    let mut ranked: Vec<(i64, f64)> = scores.into_iter().collect();
    ranked.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

    ranked
        .into_iter()
        .take(final_k)
        .filter_map(|(id, score)| {
            by_id.get(&id).map(|hit| ChunkHit {
                score,
                ..hit.clone()
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hit(id: i64, text: &str, score: f64) -> ChunkHit {
        ChunkHit {
            chunk_id: id,
            source_id: "s1".into(),
            text: text.into(),
            loc: "L1".into(),
            score,
        }
    }

    #[test]
    fn fuses_two_lists_by_chunk_id() {
        let vector = vec![hit(1, "a", 0.1), hit(2, "b", 0.2)];
        let fts = vec![hit(2, "b", -1.0), hit(3, "c", -2.0)];
        let fused = reciprocal_rank_fusion(&[&vector, &fts], 60.0, 3);
        assert_eq!(fused.len(), 3);
        assert_eq!(fused[0].chunk_id, 2);
        assert!(fused[0].score > fused[1].score);
    }

    #[test]
    fn respects_final_k() {
        let a = vec![hit(1, "a", 0.0), hit(2, "b", 0.0)];
        let b = vec![hit(3, "c", 0.0), hit(4, "d", 0.0)];
        let fused = reciprocal_rank_fusion(&[&a, &b], 60.0, 2);
        assert_eq!(fused.len(), 2);
    }
}
