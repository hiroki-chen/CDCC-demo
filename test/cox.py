import pandas as pd
import lifelines

cox_data = pd.read_parquet("./cox_data.parquet")

print("the cox_data is ", cox_data);

cph = lifelines.CoxPHFitter(penalizer=0.1)
cph.fit(cox_data, duration_col='T', event_col='event', robust=True)

print(f"Concordance Index: {cph.concordance_index_:.3f}")
print(f"Partial AIC: {cph.AIC_partial_:.1f}")
print(f"Log-likelihood ratio test: {cph.log_likelihood_ratio_test().test_statistic:.2f}")

# expected output: 0.786, 46.48
