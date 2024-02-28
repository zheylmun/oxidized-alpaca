use serde::Deserialize;

enum AcountStatus {
    /// The account is onboarding.
    Onboarding,
    /// The account application submission failed for some reason.
    SubissionFailed,
    /// The account has been submitted for review.
    Submitted,
    /// The account information is being updated.
    AccountUpdated,
    /// The final account approval is pending.
    ApprovalPending,
    /// The account is active and ready for trading.
    Active,
    /// The account application has been rejected.
    Rejected,
}
struct acount_details {
    id: String,
    account_number: String,
}
