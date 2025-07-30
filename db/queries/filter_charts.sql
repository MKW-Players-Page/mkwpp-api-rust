-- $1 - track_id
-- $2 - category
-- $3 - is_lap
-- $4 - max_date
-- $5 - region_ids
-- $6 - limit

SELECT *
FROM (
    SELECT *,
        (RANK() OVER(ORDER BY value ASC))::INTEGER AS rank,
        ((FIRST_VALUE(value) OVER(ORDER BY value ASC))::FLOAT8 / value::FLOAT8) AS prwr
    FROM (
        SELECT *
        FROM (
            SELECT
                scores.id AS s_id,
                scores.value,
                scores.category,
                scores.is_lap,
                scores.track_id,
                ROW_NUMBER() OVER(
                    PARTITION BY players.id
                    ORDER BY scores.value ASC, standard_levels.value ASC
                ) AS row_n,
                date,
                video_link,
                ghost_link,
                comment,
                was_wr,
                players.id,
                COALESCE(standard_levels.code, 'NW') AS std_lvl_code,
                name,
                alias,
                region_id FROM scores
            LEFT JOIN players ON
                scores.player_id = players.id
            LEFT JOIN standards ON
                scores.track_id = standards.track_id AND
                scores.value <= standards.value AND
                standards.category <= scores.category AND
                standards.is_lap = scores.is_lap
            LEFT JOIN standard_levels ON
                standard_levels.id = standards.standard_level_id
            WHERE
                scores.track_id = $1 AND
                scores.category <= $2 AND
                scores.is_lap = $3 AND
                scores.date <= $4 AND
                players.region_id = ANY($5) 
            ORDER BY value ASC, standard_levels.value ASC
        ) WHERE row_n = 1
    ) ORDER BY value ASC, date DESC
) WHERE rank <= $6;