-- $1 - track_id
-- $2 - category
-- $3 - is_lap

UPDATE
    scores AS og
SET
    was_wr = (
        SELECT
            inn.value = og.value
        FROM
            scores AS inn
        WHERE
            inn.track_id = og.track_id AND
            inn.category <= og.category AND
            inn.is_lap = og.is_lap AND
            inn.date <= og.date
        ORDER BY
            inn.value
        LIMIT 1
    )
WHERE
    og.track_id = $1 AND
    og.category <= $2 AND
    og.is_lap = $3