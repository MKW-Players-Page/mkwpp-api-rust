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