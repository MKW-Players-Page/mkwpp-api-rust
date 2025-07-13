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
                inn.track_id = og.track_id AND
                inn.category <= og.category AND
                inn.is_lap = og.is_lap AND
                inn.date <= og.date
        ) AS ext
        WHERE ext.id = og.id
    )