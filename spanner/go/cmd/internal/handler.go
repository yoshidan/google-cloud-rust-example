package internal

import (
	"cloud.google.com/go/spanner"
	"fmt"
	"github.com/google/uuid"
	"net/http"
	"time"
)

func errorResponse(writer http.ResponseWriter, err error ) {
	fmt.Printf("err %+v\n", err)
	writer.WriteHeader(http.StatusInternalServerError)
	_, _ = writer.Write([]byte(err.Error()))
}

func CreateUser(client *spanner.Client) http.HandlerFunc {
	return func(writer http.ResponseWriter, request *http.Request) {
		userID, err := uuid.NewRandom()
		if err != nil {
			errorResponse(writer,err)
			return
		}
		userIDString := userID.String()
		userMutation, err := spanner.InsertStruct("User", &User{
			UserId:    userIDString,
			Premium:   false,
			UpdatedAt: spanner.CommitTimestamp,
		})
		if err != nil {
			errorResponse(writer,err)
			return
		}
		ms := make([]*spanner.Mutation, 0)
		ms = append(ms, userMutation)

		for i := 0; i < 10; i ++{
			itemMutation, err := spanner.InsertStruct("UserItem", &UserItem {
				UserId:    userIDString,
				ItemId: int64(i),
				Quantity: 0,
				UpdatedAt: spanner.CommitTimestamp,
			})
			if err != nil {
				errorResponse(writer,err)
				return
			}
			ms = append(ms,itemMutation)

			characterMutation, err := spanner.InsertStruct("UserCharacter", &UserCharacter {
				UserId:    userIDString,
				CharacterId: int64(i),
				Level: 1,
				AcquiredAt: time.Now(),
				UpdatedAt: spanner.CommitTimestamp,
			})
			if err != nil {
				errorResponse(writer,err)
				return
			}
			ms = append(ms,characterMutation)
		}

		_, err = client.Apply(request.Context(),ms )
		if err != nil {
			errorResponse(writer, err)
		}
		writer.WriteHeader(http.StatusOK)
		_, _ = writer.Write([]byte(userIDString))
	}
}
