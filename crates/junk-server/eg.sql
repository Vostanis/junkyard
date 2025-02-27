SELECT 
    json_build_object(
        'prices', (
            SELECT json_agg(
                json_build_object(
                    'date', dt::DATE,
                    'perc', perc,
                    'adj_close', adj_close,
                    'adj_close_20ma', adj_close_20ma,
                    'adj_close_50ma', adj_close_50ma,
                    'adj_close_200ma', adj_close_200ma,
                    'volume', volume,
                    'volume_7ma', volume_7ma,
                    'volume_90ma', volume_90ma
                )
                ORDER BY dt::DATE DESC
            )
            FROM stock.prices_matv
            WHERE symbol = 'NVDA'
        ),
        'financials', (
            SELECT json_agg(
                json_build_object(
                    'end_date', end_date,
                    'price', price,
                    'revenue', revenue,
                    'earnings', earnings,
                    'earnings_perc', earnings_perc,
                    'eps', eps,
                    'gross_profit', gross_profit,
                    'operating_income', operating_income,
                    'accumulated_earnings', accumulated_earnings,
                    'debt_to_equity', debt_to_equity
                )
                ORDER BY end_date DESC
            )
            FROM stock.std_financials std
            INNER JOIN stock.symbols sy ON sy.pk = std.symbol_pk
            WHERE sy.symbol = 'NVDA'
        )
    ) as combined_data;