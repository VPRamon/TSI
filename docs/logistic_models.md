# Logistic Models for Scheduling Prediction

## Introduction

The **Trends** page of the TSI (Telescope Scheduling Intelligence) dashboard displays a predictive analysis based on **logistic regression with interaction terms**.
This document describes in detail what these models are, how they are trained, what information they provide, and how to interpret them from a data science perspective.

---

## 1. What Is a Logistic Regression Model?

### 1.1 Theoretical Foundation

**Logistic regression** is a supervised statistical model used for binary classification problems. Unlike linear regression, which predicts continuous values, logistic regression estimates the **probability** that an observation belongs to a specific class (in our case: “will be scheduled” vs “will not be scheduled”).

The model uses the **logistic (sigmoid)** function to transform a linear combination of predictor variables into a probability between 0 and 1:

$$
P(y=1|X) = \frac{1}{1 + e^{-(\beta_0 + \beta_1 x_1 + \beta_2 x_2 + ... + \beta_n x_n)}}
$$

Where:

* $P(y=1|X)$ is the probability that the observation will be scheduled
* $X$ represents the predictor variables (features)
* $\beta_i$ are the coefficients learned during training
* $e$ is the base of the natural logarithm

### 1.2 Advantages of Logistic Regression

1. **Interpretability**: The coefficients $\beta_i$ have a direct interpretation in terms of log-odds ratios
2. **Probabilistic output**: Produces calibrated probabilities, not just binary classifications
3. **Computational efficiency**: Fast to train and predict, even on large datasets
4. **Built-in regularization**: Supports L1/L2 regularization to prevent overfitting
5. **Statistical grounding**: Backed by robust statistical theory and decades of empirical validation

---

## 2. Logistic Regression with Interactions in TSI

### 2.1 Predictor Variables

The model uses **three main variables**:

1. **Priority**: Level of priority of the astronomical observation

   * Categorical values converted to ordinal numeric form: Low=1, Medium=2, High=3, Very High=4
   * Represents the scientific importance or urgency of the observation

2. **Visibility (Total Visibility)**: `total_visibility_hours`

   * Total hours the object is visible from the telescope
   * Continuous variable, typically in the range [0, 24] hours
   * Critical factor: visibility = 0 means physical impossibility to observe

3. **Requested Time**: `requested_hours`

   * Duration requested for the observation, in hours
   * Continuous variable that reflects the complexity or depth of the observation
   * Affects feasibility: long observations compete for limited resources

### 2.2 Interaction Terms

The model includes **interaction terms** in addition to the main variables.
Second-order interaction terms are generated using `PolynomialFeatures(degree=2, interaction_only=True)`.

**Generated features** (6 total):

1. `priority_num` – Priority (original variable)
2. `visibility` – Total visibility (original variable)
3. `requested_time` – Requested time (original variable)
4. `priority_num × visibility` – **Priority–visibility interaction**
5. `priority_num × requested_time` – **Priority–time interaction**
6. `visibility × requested_time` – **Visibility–time interaction**

#### Why are interactions important?

Interactions capture **combined non-linear effects** that independent variables cannot model:

* **Priority × Visibility**: High priority only matters when visibility exists. With visibility = 0, priority is irrelevant.
* **Visibility × Requested Time**: Long requested times are problematic only if visibility is limited; with high visibility, the impact is smaller.
* **Priority × Requested Time**: Long, high-priority observations may receive special scheduling treatment.

### 2.3 Data Preprocessing

Before training, critical transformations are applied:

#### a) Standardization (StandardScaler)

All numeric variables are standardized as:

$$
x_{scaled} = \frac{x - \mu}{\sigma}
$$

Where $\mu$ is the mean and $\sigma$ is the standard deviation of each feature.

**Benefits**:

* Puts all variables on the same scale
* Improves convergence of the optimization algorithm (L-BFGS)
* Enables comparison of coefficient magnitudes

#### b) Exclusion of Visibility = 0 (optional)

By default, observations with `total_visibility_hours = 0` are excluded from training:

```python
if exclude_zero_visibility:
    df_model = df_model[df_model[visibility_col] > 0]
```

**Scientific justification**:

* Observations with visibility = 0 **can never be scheduled** (physical restriction)
* Including them would bias the model toward trivial patterns
* The model should learn meaningful relationships within the feasible range (visibility > 0)

**Note**: These observations are still shown in empirical analyses for transparency.

#### c) Class Weighting

The default is `class_weight='balanced'`:

$$
w_i = \frac{n_{total}}{n_{classes} \times n_{samples_in_class_i}}
$$

**Purpose**:

* Compensates for class imbalance (scheduled vs not scheduled)
* Prevents the model from always predicting the majority class
* Particularly useful when the scheduling rate is very high or low

---

## 3. Training Process

### 3.1 Scikit-learn Pipeline

The model is implemented as a **Scikit-learn Pipeline** with three sequential stages:

```python
pipeline = Pipeline([
    ('scaler', StandardScaler()),                    
    ('poly', PolynomialFeatures(degree=2, interaction_only=True, include_bias=False)),
    ('classifier', LogisticRegression(max_iter=500, class_weight='balanced',
                                      random_state=42, solver='lbfgs'))
])
```

**Pipeline advantages**:

* Encapsulates preprocessing and modeling
* Prevents data leakage (the scaler fits only on training data)
* Simplifies prediction on new data

### 3.2 Optimization Algorithm

Uses the **L-BFGS** (Limited-memory Broyden–Fletcher–Goldfarb–Shanno) solver:

* **Type**: Second-order (quasi-Newton) optimization
* **Advantages**: Fast convergence, handles multiple features well
* **Objective function**: Minimizes log-loss (cross-entropy) with L2 regularization

$$
\mathcal{L}(\beta) = -\frac{1}{N}\sum_{i=1}^{N} [y_i \log(p_i) + (1-y_i)\log(1-p_i)] + \lambda||\beta||_2^2
$$

### 3.3 Validation and Minimum Requirements

Robust validation checks are implemented:

```python
if len(df_model) < 20:
    raise ValueError("Insufficient data to train model: {len(df_model)} rows (minimum 20 required)")
if len(unique_classes) < 2:
    raise ValueError(f"At least 2 classes required in target. Classes found: {unique_classes}")
```

**Production data summary** (`training_summary.json`):

* Train set: 337 observations (99.7% scheduled)
* Validation set: 112 observations (91.1% scheduled)
* Test set: 114 observations (49.1% scheduled)

---

## 4. Evaluation Metrics

The model reports four main metrics:

### 4.1 Accuracy

$$
\text{Accuracy} = \frac{TP + TN}{TP + TN + FP + FN}
$$

* **Definition**: Proportion of correct predictions
* **Range**: [0, 1], where 1 is perfect
* **Limitation**: Can be misleading for imbalanced classes
* **Usage**: Calculated on the full training set

### 4.2 AUC-ROC (Area Under the Curve)

$$
\text{AUC} = \int_0^1 \text{TPR}(t) , d\text{FPR}(t)
$$

* **Definition**: Area under the ROC curve
* **Range**: [0, 1], 0.5 = random, 1 = perfect
* **Interpretation**: Probability that a scheduled observation is ranked higher than an unscheduled one
* **Test value**: **0.958** (excellent discrimination)
* **Advantage**: Independent of decision threshold

### 4.3 Precision–Recall AUC

* **Test value**: **0.958**
* **Use**: Especially useful for imbalanced datasets
* **Interpretation**: Trade-off between precision and recall across thresholds

### 4.4 Brier Score

$$
\text{Brier} = \frac{1}{N}\sum_{i=1}^{N}(p_i - y_i)^2
$$

* **Definition**: Mean squared error of predicted probabilities
* **Range**: [0, 1], 0 = perfect
* **Test value**: **0.173** (good calibration)
* **Interpretation**: Measures how well probabilities are calibrated

### 4.5 Additional Dashboard Metrics

* **n_samples**: Number of training observations (post-filtering)
* **n_scheduled**: Number of scheduled observations in training
* **Feature count**: 6 including interactions

---

## 5. Interpretation of Results

### 5.1 Probabilistic Predictions

The model outputs **calibrated probabilities** between 0 and 1:

```python
y_pred_proba = pipeline.predict_proba(X)[:, 1]
```

**Interpretation**:

* `proba = 0.95`: 95% probability of being scheduled
* `proba = 0.30`: 30% probability of being scheduled
* `proba ≈ 0.00`: Cases with visibility = 0 or highly unfavorable combinations

### 5.2 Prediction Curves vs Visibility

The dashboard displays curves showing how the estimated probability changes with visibility, **separated by priority level**:

**Procedure**:

1. Fix the requested time (adjustable via sidebar)
2. Create a visibility grid [min, max] with 100 points
3. Evaluate probabilities across the grid for each priority level
4. Plot multiple curves (one per priority)

**Expected patterns**:

* **Increasing curves**: Higher visibility → higher probability
* **Vertical separation**: Higher priorities → higher curves
* **Saturation effect**: Curves flatten at very high visibility
* **Visible interactions**: Slope varies by priority (interaction effect)

### 5.3 Effects of Interactions

**Example 1: Priority × Visibility**

* Without interaction: priority effect constant regardless of visibility
* With interaction: priority impact grows when visibility is moderate/high; with zero visibility, priority is irrelevant

**Example 2: Visibility × Requested Time**

* Without interaction: requested time effect independent of visibility
* With interaction: long requests are more problematic under low visibility

---

## 6. Model Applications

### 6.1 Predictive Analysis

* **Identify high-risk observations**: Probability < 0.3
* **Prioritize scheduling**: Focus on mid-range probabilities (0.4–0.7)
* **Validate scheduling policies**: Compare predictions to outcomes

### 6.2 “What-if” Analysis

The model supports counterfactual exploration:

* “What if we increased the observation’s priority?”
* “How does the probability change if we request less time?”
* “What minimum visibility is needed for a 70% chance?”

### 6.3 Insights for Scheduling Policies

* **Priority sensitivity**: Assess whether priority rules are respected
* **Visibility constraints**: Quantify cost of limited visibility
* **Time–probability trade-offs**: Guide duration-related decisions

---

## 7. Limitations and Considerations

### 7.1 Model Assumptions

1. **Linearity in transformed space**: Logistic regression assumes log-odds is linear in features (including interactions)
2. **Independence of observations**: Each observation assumed independent (may be violated by temporal correlations)
3. **Non-causality**: Captures associations, not causal effects. Correlation ≠ causation

### 7.2 Known Limitations

1. **Potential overfitting**: Low risk with 6 features and balanced weights, but possible under extreme imbalance
2. **Higher-order interactions**: Only pairwise (second-order) interactions included
3. **Omitted variables**: Weather, instrument availability, etc., not modeled
4. **Single decision threshold**: Typically 0.5 but adjustable

### 7.3 Temporal Validation

**Important**: The model is trained and evaluated on a specific time period.
For production:

* Use **temporal cross-validation** (preserving chronological order)
* Monitor **model drift** over time
* **Retrain periodically** with recent data

---

## 8. Comparison with Other Models

According to `training_summary.json`, several algorithms were tested; logistic regression was selected as the **best_model**.
Other models included:

* **Random Forest** (tree ensemble)
* **Gradient Boosting** (iterative boosting algorithms)

**Why Logistic Regression was chosen**:

1. **Interpretability**: Direct coefficient meaning
2. **Calibration**: Better-calibrated probabilities than tree-based models
3. **Efficiency**: Faster real-time inference
4. **Regularization**: Explicit control over overfitting (L2)

---

## 9. Technical Implementation

### 9.1 Code Location

* **Main module**: `/src/tsi/modeling/trends.py`
* **Training function**: `fit_logistic_with_interactions()`
* **Prediction function**: `predict_probs()`
* **Grid generation**: `create_prediction_grid()`

### 9.2 Dashboard Page

* **File**: `/src/tsi/pages/scheduling_trends.py`
* **Section**: “4️⃣ Logistic model with interactions”
* **Caching**: Retrains only when filters change (`@st.cache_resource`)

### 9.3 Unit Tests

* **File**: `/tests/tsi/modeling/test_trends.py`
* **Coverage**: Training, prediction, error validation, exclusion of visibility = 0

---

## 10. Conclusions

The logistic regression model with interactions implemented on the **Trends** page provides:

✅ **Calibrated probability predictions** for scheduling
✅ **Interpretability** via coefficients and visualizations
✅ **Combined effect capture** through interaction terms
✅ **Rigorous validation** with multiple metrics (Accuracy, AUC, Brier Score)
✅ **Computational efficiency** for interactive dashboard analysis

**Recommended next steps**:

1. Temporal validation using multiple scheduling iterations
2. Feature importance analysis for contribution quantification
3. Further probability calibration (Platt scaling or isotonic regression)
4. Evaluation of non-linear models (XGBoost, Neural Networks) for comparison
5. Inclusion of additional features (weather, instruments, etc.)

---

## Technical References

* **Scikit-learn Documentation**: [Logistic Regression](https://scikit-learn.org/stable/modules/generated/sklearn.linear_model.LogisticRegression.html)
* **PolynomialFeatures**: [Feature Engineering](https://scikit-learn.org/stable/modules/generated/sklearn.preprocessing.PolynomialFeatures.html)
* **Pipeline**: [ML Pipelines](https://scikit-learn.org/stable/modules/generated/sklearn.pipeline.Pipeline.html)
* **Training Artifacts**: `/src/tsi/modeling/artifacts/training_summary.json`
