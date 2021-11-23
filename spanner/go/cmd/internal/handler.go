package internal

import (
	"cloud.google.com/go/spanner"
	"context"
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

func ReadInventory(client *spanner.Client) http.HandlerFunc {
	return func(writer http.ResponseWriter, request *http.Request) {
		userID := request.FormValue("user_id")
		stmt := spanner.NewStatement(`SELECT * ,
			ARRAY (SELECT AS STRUCT * FROM UserItem WHERE UserId = @Param1 ) AS UserItem, 
			ARRAY (SELECT AS STRUCT * FROM UserCharacter WHERE UserId = @Param1 ) AS UserCharacter  
			FROM User 
			WHERE UserId = @Param1
		`)
		stmt.Params["Param1"] = userID
		tx := client.Single()
		iter := tx.Query(request.Context(), stmt)
		defer iter.Stop()
		row, err := iter.Next()
		if err != nil {
			errorResponse(writer,err)
			return
		}
		var userId string
		if err = row.ColumnByName("UserId", &userId); err != nil {
			errorResponse(writer,err)
			return
		}
		var userItems []*UserItem
		if err = row.ColumnByName("UserItem", &userItems); err != nil {
			errorResponse(writer,err)
			return
		}

		var userCharacters []*UserCharacter
		if err = row.ColumnByName("UserCharacter", &userCharacters); err != nil {
			errorResponse(writer,err)
			return
		}
		writer.WriteHeader(http.StatusOK)
		_, _ = writer.Write([]byte(fmt.Sprintf("user=%d, item=%d, character=%d", userId, len(userItems), len(userCharacters))))
	}
}

func UpdateInventory(client *spanner.Client) http.HandlerFunc {
	return func(writer http.ResponseWriter, request *http.Request) {
		userID := request.FormValue("user_id")

		cts, err := client.ReadWriteTransaction(request.Context(), func(ctx context.Context, transaction *spanner.ReadWriteTransaction) error {
			stmt := spanner.NewStatement("SELECT * From UserItem WHERE UserId = @UserId")
			stmt.Params["UserId"] = userID
			iter := transaction.Query(ctx, stmt)
			ms := make([]*spanner.Mutation,0)
			for {
				row ,err := iter.Next()
				if err != nil {
					break
				}
				var itemId int64
				if err = row.ColumnByName("ItemId", &itemId); err != nil {
					return err
				}
				var quantity int64
				if err = row.ColumnByName("Quantity", &quantity); err != nil {
					return err
				}
				ms = append(ms, spanner.Update("UserItem", []string{"UserId", "ItemId", "Quantity"}, []interface{}{userID, itemId, quantity + 1}))
			}
			if err := transaction.BufferWrite(ms); err != nil {
				return err
			}
			return nil
		})

		if err != nil {
			errorResponse(writer,err)
			return
		}
		writer.WriteHeader(http.StatusOK)
		_, _ = writer.Write([]byte(fmt.Sprintf("ts=%d", cts.Second())))
	}
}
