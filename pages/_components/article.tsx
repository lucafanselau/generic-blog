import React, {FC} from "react";

export const Article: FC = ({children}) => {

    return <div className={"prose"}>{children}</div>;
}