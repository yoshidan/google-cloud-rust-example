package internal

import "time"

type User struct {
	UserId string `spanner:"UserId"`
	Premium bool `spanner:"Premium"`
	UpdatedAt time.Time `spanner:"UpdatedAt"`
}

type UserItem struct {
	UserId string `spanner:"UserId"`
	ItemId int64 `spanner:"ItemId"`
	Quantity int64 `spanner:"Quantity"`
	UpdatedAt time.Time `spanner:"UpdatedAt"`
}

type UserCharacter struct {
	UserId string `spanner:"UserId"`
	CharacterId int64 `spanner:"CharacterId"`
	Level int64 `spanner:"Level"`
	AcquiredAt time.Time `spanner:"AcquiredAt"`
	UpdatedAt time.Time `spanner:"UpdatedAt"`
}