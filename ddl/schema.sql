CREATE TABLE User (
    UserId STRING(36) NOT NULL,
    Premium BOOL NOT NULL,
    UpdatedAt TIMESTAMP NOT NULL OPTIONS(allow_commit_timestamp=true)
) PRIMARY KEY(UserId);

CREATE TABLE UserItem (
    UserId STRING(36) NOT NULL,
    ItemId INT64 NOT NULL,
    Quantity INT64 NOT NULL,
    UpdatedAt TIMESTAMP NOT NULL OPTIONS(allow_commit_timestamp=true)
) PRIMARY KEY(UserId, ItemId), INTERLEAVE IN PARENT User ON DELETE CASCADE;

CREATE TABLE UserCharacter (
    UserId STRING(36) NOT NULL,
    CharacterId INT64 NOT NULL,
    Level INT64 NOT NULL,
    UpdatedAt TIMESTAMP NOT NULL OPTIONS(allow_commit_timestamp=true)
) PRIMARY KEY(UserId, CharacterId), INTERLEAVE IN PARENT User ON DELETE CASCADE;