-- $1 - track_id
-- $2 - category
-- $3 - is_lap

UPDATE
    scores AS og
SET
    initial_rank = (
        SELECT
            rank
        FROM (
            SELECT
                RANK() OVER (
                    ORDER BY value ASC, date DESC
                ) AS rank,
                inn.id
            FROM
                scores AS inn
            WHERE
                inn.track_id = $1 AND
                inn.category <= $2 AND
                inn.is_lap = $3 AND
                inn.date <= og.date
        ) AS ext
        WHERE ext.id = og.id
    )
    WHERE
        og.track_id = $1 AND
        og.category <= $2 AND
        og.is_lap = $3