-- $1 - limit
-- $2 - only_records

SELECT
    scores.id AS s_id,
    value, category,
    is_lap, track_id,
    date, players.id, name,
    alias, region_id
FROM scores
LEFT JOIN players ON scores.player_id = players.id
WHERE
    date IS NOT NULL AND
    (
        ($2 = TRUE AND initial_rank = 1) OR
        ($2 = FALSE)
    )
ORDER BY
    date DESC,
    player_id ASC,
    initial_rank ASC,
    track_id ASC,
    category ASC,
    is_lap ASC
LIMIT $1;