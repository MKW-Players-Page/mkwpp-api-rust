-- $1 - track_id
-- $2 - category
-- $3 - is_lap
-- $4 - region_ids

SELECT DISTINCT
    scores.date
FROM
    scores
LEFT JOIN players ON
    scores.player_id = players.id
WHERE
    scores.track_id = $1 AND
    scores.category <= $2 AND
    scores.is_lap = $3 AND
    players.region_id = ANY($4)
ORDER BY
    date ASC
    