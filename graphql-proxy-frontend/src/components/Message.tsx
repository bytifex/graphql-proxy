import { ReactElement } from "react";
import MessageData from "../model/message_data";

export default function Message(props: {
	message: MessageData,
}): ReactElement {
	return <div>Message {props.message}</div>;
}
