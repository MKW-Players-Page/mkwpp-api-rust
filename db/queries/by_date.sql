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
        ($2 = TRUE AND was_wr = TRUE) OR
        ($2 = FALSE)
    )
ORDER BY
    date DESC,
    player_id ASC,
    was_wr ASC,
    track_id ASC,
    category ASC,
    is_lap ASC
LIMIT $1;