package bot

import (
	"context"
	"github.com/Light-Keeper/ir-remote/internal/irremote"
	tgbotapi "github.com/go-telegram-bot-api/telegram-bot-api"
	"strconv"
	"strings"
)

type Bot struct {
	session            *irremote.Session
	api                *tgbotapi.BotAPI
	botAuthorizedUsers []int
}

func NewBot(apikey string, botAuthorizedUsers string, session *irremote.Session) *Bot {
	api, err := tgbotapi.NewBotAPI(apikey)
	if err != nil {
		panic(err)
	}

	return &Bot{
		session:            session,
		api:                api,
		botAuthorizedUsers: parseAuthorizedUsers(botAuthorizedUsers),
	}
}

func parseAuthorizedUsers(botAuthorizedUsers string) []int {
	var result []int
	for _, user := range strings.Split(botAuthorizedUsers, ",") {
		intUserId, err := strconv.Atoi(user)
		if err != nil {
			panic(err)
		}
		result = append(result, intUserId)
	}
	return result
}

func (b *Bot) Run(ctx context.Context) error {
	u := tgbotapi.NewUpdate(0)
	u.Timeout = 60
	updates, err := b.api.GetUpdatesChan(u)

	if err != nil {
		return err
	}

	for {
		select {
		case <-ctx.Done():
			b.api.StopReceivingUpdates()
			return nil

		case update := <-updates:
			if update.Message == nil {
				continue
			}
			if !b.isAuthorized(update.Message.From.ID) {
				b.api.Send(tgbotapi.NewMessage(update.Message.Chat.ID, "Вы не авторизованы"))
				continue
			}

			if update.Message.IsCommand() && update.Message.Command() == "start" {
				b.respond(ctx, update.Message.Chat.ID, "Привет! Я бот для управления кондиционером. Нажимай кнопки, и я буду отправлять команды пульту в кондиционер")
				continue
			}

			chatId := update.Message.Chat.ID
			handler := lookupHandler(update.Message.Text)
			handler(b, ctx, chatId)
		}
	}
}

var buttons = [][]struct {
	label   string
	handler func(b *Bot, ctx context.Context, chatId int64)
}{
	{
		{"🔴выкл", handleButtonOff},
		{"🥶+23", handleButtonCold22},
		{"💧+23", handleUnknown},
	},
	{
		{"? статус", handleButtonStatus},
		{"🥶+20", handleUnknown},
		{"💧+20", handleUnknown},
	},
}

var customKeyboard tgbotapi.ReplyKeyboardMarkup

func init() {
	customKeyboard = tgbotapi.ReplyKeyboardMarkup{
		Keyboard: [][]tgbotapi.KeyboardButton{},
	}

	for _, row := range buttons {
		var keyboardRow []tgbotapi.KeyboardButton
		for _, button := range row {
			keyboardRow = append(keyboardRow, tgbotapi.NewKeyboardButton(button.label))
		}
		customKeyboard.Keyboard = append(customKeyboard.Keyboard, keyboardRow)
	}
}

func lookupHandler(text string) func(b *Bot, ctx context.Context, chatId int64) {
	for _, row := range buttons {
		for _, button := range row {
			if button.label == text {
				return button.handler
			}
		}
	}
	return handleUnknown
}

func handleButtonOff(b *Bot, ctx context.Context, chatId int64) {
	b.sendCommandAndReplay(ctx, commandOff, chatId)
}

func handleButtonCold22(b *Bot, ctx context.Context, chatId int64) {
	b.sendCommandAndReplay(ctx, commandCold22, chatId)
}

func handleButtonWater22(b *Bot, ctx context.Context, chatId int64) {
	b.sendCommandAndReplay(ctx, commandWater22, chatId)
}

func handleButtonStatus(b *Bot, ctx context.Context, chatId int64) {
	b.respond(ctx, chatId, "")
}

func handleUnknown(b *Bot, ctx context.Context, chatId int64) {
	b.respond(ctx, chatId, "Неизвестная команда")
}

func (b *Bot) sendCommandAndReplay(ctx context.Context, command []int, chatId int64) {
	err := b.session.SendCommand(ctx, command)
	if err != nil {
		b.respond(ctx, chatId, "Error: "+err.Error())
	} else {
		b.respond(ctx, chatId, "Команда успешно отправлена (но неизвестно, принята ли она кондиционером)")
	}
}

func (b *Bot) respond(_ context.Context, chatId int64, text string) {
	var statusMessage string
	if b.session.IsOnline() {
		statusMessage = "Статус пульта: 🟢онлайн"
	} else {
		statusMessage = "Статус пульта: 🚫недоступен"
	}

	text += "\n" + statusMessage
	message := tgbotapi.NewMessage(chatId, text)
	message.ReplyMarkup = customKeyboard

	_, err := b.api.Send(message)
	if err != nil {
		println(err.Error())
	}
}

func (b *Bot) isAuthorized(id int) bool {
	for _, user := range b.botAuthorizedUsers {
		if user == id {
			return true
		}
	}
	return false
}
